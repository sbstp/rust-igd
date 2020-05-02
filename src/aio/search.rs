use std::collections::HashMap;
use std::net::SocketAddr;
use std::pin::Pin;
use std::str;
use std::task::{Context, Poll};

use futures::prelude::*;
use futures::Future;

use hyper::Client;

use tokio::net::UdpSocket;
use tokio::time::timeout;

use hyper::body::Bytes;

use crate::aio::Gateway;
use crate::common::{messages, parsing, SearchOptions};
use crate::errors::SearchError;

const MAX_RESPONSE_SIZE: usize = 1500;

/// Search for a gateway with the provided options
pub async fn search_gateway(options: SearchOptions) -> Result<Gateway, SearchError> {
    // Create socket for future calls
    let socket = UdpSocket::bind(&options.bind_addr).await?;

    let search_future = SearchFuture::search(socket, options.broadcast_address).await?;
    // Create future and issue request
    match options.timeout {
        Some(t) => timeout(t, search_future).await?,
        None => search_future.await,
    }
}

pub struct SearchFuture {
    socket: UdpSocket,
    pending: HashMap<SocketAddr, SearchState>,
}

enum SearchState {
    Connecting(Pin<Box<dyn Future<Output = Result<Bytes, SearchError>> + Send>>),
    Done(String),
    Error,
}

impl SearchFuture {
    // Create a new search
    async fn search(mut socket: UdpSocket, addr: SocketAddr) -> Result<SearchFuture, SearchError> {
        debug!(
            "sending broadcast request to: {} on interface: {:?}",
            addr,
            socket.local_addr()
        );
        socket.send_to(messages::SEARCH_REQUEST.as_bytes(), &addr).await?;
        Ok(SearchFuture {
            socket,
            pending: HashMap::new(),
        })
    }

    // Handle a UDP response message
    fn handle_broadcast_resp(from: SocketAddr, data: &[u8]) -> Result<(SocketAddr, String), SearchError> {
        debug!("handling broadcast response from: {}", from);

        // Convert response to text
        let text = str::from_utf8(&data).map_err(|e| SearchError::from(e))?;

        // Parse socket address and path
        let (addr, path) = parsing::parse_search_result(text)?;

        Ok((SocketAddr::V4(addr), path))
    }

    // Issue a control URL request over HTTP using the provided
    fn request_control_url(
        addr: SocketAddr,
        path: String,
    ) -> Result<Pin<Box<dyn Future<Output = Result<Bytes, SearchError>> + Send>>, SearchError> {
        let client = Client::new();

        let uri = match format!("http://{}{}", addr, path).parse() {
            Ok(uri) => uri,
            Err(err) => return Err(SearchError::from(err)),
        };

        debug!("requesting control url from: {}", uri);

        Ok(Box::pin(
            client
                .get(uri)
                .and_then(|resp| hyper::body::to_bytes(resp.into_body()))
                .map_err(|e| SearchError::from(e)),
        ))
    }

    // Process a control response to extract the control URL
    fn handle_control_resp(addr: SocketAddr, resp: Bytes) -> Result<String, SearchError> {
        debug!("handling control response from: {}", addr);

        // Create a cursor over the response data
        let c = std::io::Cursor::new(&resp);

        // Parse control URL out of body
        let url = parsing::parse_control_url(c)?;

        Ok(url)
    }
}

impl Future for SearchFuture {
    type Output = Result<Gateway, SearchError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<Gateway, SearchError>> {
        // Poll for (and handle) incoming messages
        let mut buff = [0u8; MAX_RESPONSE_SIZE];
        let resp = self.socket.poll_recv_from(cx, &mut buff);
        if let Poll::Ready(Ok((n, from))) = resp {
            // Try handle response messages
            if let Ok((addr, path)) = Self::handle_broadcast_resp(from, &buff[0..n]) {
                if !self.pending.contains_key(&addr) {
                    debug!("received broadcast response from: {}", from);

                    // Issue control request
                    match Self::request_control_url(addr, path) {
                        // Store pending requests
                        Ok(f) => {
                            self.pending.insert(addr, SearchState::Connecting(f));
                        }
                        Err(e) => return Poll::Ready(Err(e)),
                    }
                } else {
                    debug!("received duplicate broadcast response from: {}, dropping", from);
                }
            }
        }
        if let Poll::Ready(Err(err)) = resp {
            return Poll::Ready(Err(err.into()));
        }

        // Poll on any outstanding control requests
        for (addr, state) in &mut self.pending {
            // Poll if we're in the connecting state
            let resp = {
                let c = match state {
                    SearchState::Connecting(c) => c,
                    _ => continue,
                };

                match c.as_mut().poll(cx)? {
                    Poll::Ready(resp) => resp,
                    _ => continue,
                }
            };

            // Handle any responses
            if let Ok(control_url) = Self::handle_control_resp(*addr, resp) {
                debug!("received control url from: {} (url: {})", addr, control_url);
                *state = SearchState::Done(url.clone());

                match addr {
                    SocketAddr::V4(a) => {
                        let g = Gateway {
                            addr: *a,
                            control_url,
                        };
                        return Poll::Ready(Ok(g));
                    }
                    _ => warn!("unsupported IPv6 gateway response from addr: {}", addr),
                }
            } else {
                *state = SearchState::Error;
            }
        }

        Poll::Pending
    }
}
