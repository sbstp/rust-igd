use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::str;
use std::time::Duration;

use futures::future;
use futures::{Future, IntoFuture, Stream};
use hyper::Client;

use tokio::prelude::FutureExt;
use tokio::net::UdpSocket;

use async::Gateway;
use common::{messages, parsing};
use errors::SearchError;

/// Search gateway, bind to all interfaces and use a timeout of 3 seconds.
///
/// Bind to all interfaces.
/// The request will timeout after 3 seconds.
pub fn search_gateway() -> Box<Future<Item = Gateway, Error = SearchError> + Send> {
    search_gateway_timeout(Duration::from_secs(3))
}

/// Search gateway, bind to all interfaces and use the given duration for the timeout.
///
/// Bind to all interfaces.
/// The request will timeout after the given duration.
pub fn search_gateway_timeout(timeout: Duration) -> Box<Future<Item = Gateway, Error = SearchError> + Send> {
    search_gateway_from_timeout(Ipv4Addr::new(0, 0, 0, 0), timeout)
}

/// Search gateway, bind to the given interface and use a time of 3 seconds.
///
/// Bind to the given interface.
/// The request will timeout after 3 seconds.
pub fn search_gateway_from(ip: Ipv4Addr) -> Box<Future<Item = Gateway, Error = SearchError> + Send> {
    search_gateway_from_timeout(ip, Duration::from_secs(3))
}

/// Search gateway, bind to the given interface and use the given duration for the timeout.
///
/// Bind to the given interface.
/// The request will timeout after the given duration.
pub fn search_gateway_from_timeout(
    ip: Ipv4Addr,
    timeout: Duration,
) -> Box<Future<Item = Gateway, Error = SearchError> + Send> {
    let addr = SocketAddr::V4(SocketAddrV4::new(ip, 0));
    let task = UdpSocket::bind(&addr)
        .into_future()
        .and_then(|socket| socket.send_dgram(messages::SEARCH_REQUEST.as_bytes(), &"239.255.255.250:1900".parse().unwrap()))
        .and_then(|(socket, _)| socket.recv_dgram(vec![0u8; 1500]))
        .map_err(|err| SearchError::from(err))
        .and_then(|(_sock, buf, n, _addr)| {

            str::from_utf8(&buf[..n])
                .map_err(|err| SearchError::from(err))
                .and_then(|text| {
                    println!("Recv: {:?}", text);
                    parsing::parse_search_result(text)
                })
        })
        .and_then(move |location| {
            get_control_url(&location)
                .and_then(move |control_url| Ok(Gateway::new(location.0, control_url)))
        });
    let timeout = task.timeout(timeout)
        .map_err(|e| e.into() );

    Box::new(timeout)
}

fn get_control_url(
    location: &(SocketAddrV4, String),
) -> Box<Future<Item = String, Error = SearchError> + Send> {
    let client = Client::new();
    let uri = match format!("http://{}{}", location.0, location.1).parse() {
        Ok(uri) => uri,
        Err(err) => return Box::new(future::err(SearchError::from(err))),
    };
    let future = client
        .get(uri)
        .and_then(|resp| resp.into_body().concat2())
        .then(|result| match result {
            Ok(body) => parsing::parse_control_url(body.as_ref()),
            Err(err) => Err(SearchError::from(err)),
        });
    Box::new(future)
}
