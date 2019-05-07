use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::str;
use std::time::Duration;
use std::collections::HashMap;

use futures::prelude::*;
use futures::future::Either;
use futures::{Future, Stream};

use hyper::Client;

use tokio::prelude::FutureExt;
use tokio::net::UdpSocket;

use bytes::Bytes;

use async::Gateway;
use common::{messages, parsing};
use errors::SearchError;

const MAX_RESPONSE_SIZE: usize = 1500;

/// Gateway search configuration
/// SearchOptions::default() should suffice for most situations
pub struct SearchOptions {
    /// Bind address for UDP socket (defaults to all interfaces)
    pub bind_addr: SocketAddr,
    /// Broadcast address for discovery packets
    pub broadcast_address: SocketAddr,
    /// Timeout for a search iteration
    pub timeout: Option<Duration>,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            bind_addr: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0)),
            broadcast_address: "239.255.255.250:1900".parse().unwrap(),
            timeout: Some(Duration::from_secs(3)),
        }
    }
}

/// Search for a gateway with the provided options
pub fn search_gateway(options: SearchOptions) -> impl Future<Item=Gateway, Error=SearchError> {

    // Create socket for future calls
    let socket = UdpSocket::bind(&options.bind_addr).unwrap();

    // Create future and issue request
    match options.timeout {
        Some(t) => Either::A(SearchFuture::search(socket, options.broadcast_address)
            .and_then(|search| search ).timeout(t).map_err(|e| SearchError::from(e) )),
        _ => Either::B(SearchFuture::search(socket, options.broadcast_address).and_then(|search| search )),
    }
}

pub struct SearchFuture {
    socket: UdpSocket,
    pending: HashMap<SocketAddr, SearchState>,
}

enum SearchState {
    Connecting(Box<Future<Item=Bytes, Error=SearchError> + Send>),
    Done(String),
    Error,
}

impl SearchFuture {
    // Create a new search
    fn search(socket: UdpSocket, addr: SocketAddr) -> impl Future<Item=SearchFuture, Error=SearchError> {
        debug!("sending broadcast request to: {} on interface: {:?}", addr, socket.local_addr());

        socket.send_dgram(messages::SEARCH_REQUEST.as_bytes(), &addr)
            .map(|(socket, _n)| SearchFuture{socket, pending: HashMap::new() })
            .map_err(|e| SearchError::from(e) )
    }

    // Handle a UDP response message
    fn handle_broadcast_resp(from: SocketAddr, data: &[u8]) -> Result<(SocketAddr, String), SearchError> {
        debug!("handling broadcast response from: {}, data: {:?}", from, data);

        // Convert response to text
        let text = str::from_utf8(&data)
            .map_err(|e| SearchError::from(e))?;
        
        // Parse socket address and path
        let (addr, path) = parsing::parse_search_result(text)?;

        Ok((SocketAddr::V4(addr), path))
    }

    // Issue a control URL request over HTTP using the provided 
    fn request_control_url(addr: SocketAddr, path: String) -> Result<Box<Future<Item=Bytes, Error=SearchError> + Send>, SearchError> {
        let client = Client::new();

        let uri = match format!("http://{}{}", addr, path).parse() {
            Ok(uri) => uri,
            Err(err) => return Err(SearchError::from(err)),
        };

        debug!("requesting control url from: {}", uri);
        
        Ok(Box::new(client.get(uri)
            .and_then(|resp| resp.into_body().concat2() )
            .map(|chunk| chunk.into_bytes() )
            .map_err(|e| SearchError::from(e) )
        ))
    }

    // Process a control response to extract the control URL
    fn handle_control_resp(addr: SocketAddr, resp: Bytes) -> Result<String, SearchError> {
        debug!("handling control response from: {}, data: {:?}", addr, resp);

        // Create a cursor over the response data
        let c = std::io::Cursor::new(&resp);

        // Parse control URL out of body
        let url = parsing::parse_control_url(c)?;

        Ok(url)
    }
}


impl Future for SearchFuture {
    type Item=Gateway;
    type Error=SearchError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {

        // Poll for (and handle) incoming messages
        let mut buff = [0u8; MAX_RESPONSE_SIZE];
        if let Async::Ready((n, from)) = self.socket.poll_recv_from(&mut buff)? {
            // Try handle response messages
            if let Ok((addr, path)) = Self::handle_broadcast_resp(from, &buff[0..n]) {
                if !self.pending.contains_key(&addr) {
                    debug!("received broadcast response from: {}", from);

                    // Issue control request
                    let req = Self::request_control_url(addr, path)?;
                    // Store pending requests
                    self.pending.insert(addr, SearchState::Connecting(req));
                } else {
                    debug!("received duplicate broadcast response from: {}, dropping", from);
                }
            }
        }

        // Poll on any outstanding control requests
        for (addr, state) in &mut self.pending {
            // Poll if we're in the connecting state
            let resp = {
                let c = match state {
                    SearchState::Connecting(c) => c,
                    _ => continue,
                };

                match c.poll()? {
                    Async::Ready(resp) => resp,
                    _ => continue,
                }
            };

            // Handle any responses
            if let Ok(url) = Self::handle_control_resp(*addr, resp) {
                debug!("received control url from: {} (url: {})", addr, url);
                *state = SearchState::Done(url.clone());

                match addr {
                    SocketAddr::V4(a) => {
                        let g = Gateway::new(*a, url);
                        return Ok(Async::Ready(g));
                    }
                    _ => warn!("unsupported IPv6 gateway response from addr: {}", addr),
                }
                
            } else {
                *state = SearchState::Error;
            }
        }

        Ok(Async::NotReady)
    }
}
