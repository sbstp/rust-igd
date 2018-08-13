use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::str;
use std::time::Duration;

use futures::{Future, IntoFuture, Stream};
use futures::future;
use tokio_core::reactor::Handle;
use tokio_core::net::UdpSocket;
use tokio_timer::Timer;
use hyper;
use quick_xml::{Reader, events::Event};

use async::Gateway;
use errors::SearchError;
use search::{parse_result, SEARCH_REQUEST};

/// Search gateway, bind to all interfaces and use a timeout of 3 seconds.
///
/// Bind to all interfaces.
/// The request will timeout after 3 seconds.
pub fn search_gateway(handle: &Handle) -> Box<Future<Item = Gateway, Error = SearchError>> {
    search_gateway_timeout(Duration::from_secs(3), handle)
}

/// Search gateway, bind to all interfaces and use the given duration for the timeout.
///
/// Bind to all interfaces.
/// The request will timeout after the given duration.
pub fn search_gateway_timeout(
    timeout: Duration,
    handle: &Handle,
) -> Box<Future<Item = Gateway, Error = SearchError>> {
    search_gateway_from_timeout(Ipv4Addr::new(0, 0, 0, 0), timeout, handle)
}

/// Search gateway, bind to the given interface and use a time of 3 seconds.
///
/// Bind to the given interface.
/// The request will timeout after 3 seconds.
pub fn search_gateway_from(
    ip: Ipv4Addr,
    handle: &Handle,
) -> Box<Future<Item = Gateway, Error = SearchError>> {
    search_gateway_from_timeout(ip, Duration::from_secs(3), handle)
}

/// Search gateway, bind to the given interface and use the given duration for the timeout.
///
/// Bind to the given interface.
/// The request will timeout after the given duration.
pub fn search_gateway_from_timeout(
    ip: Ipv4Addr,
    timeout: Duration,
    handle: &Handle,
) -> Box<Future<Item = Gateway, Error = SearchError>> {
    let addr = SocketAddr::V4(SocketAddrV4::new(ip, 0));
    let handle = handle.clone();
    let task = UdpSocket::bind(&addr, &handle)
        .into_future()
        .and_then(|socket| {
            socket.send_dgram(
                SEARCH_REQUEST.as_bytes(),
                "239.255.255.250:1900".parse().unwrap(),
            )
        })
        .and_then(|(socket, _)| socket.recv_dgram(vec![0u8; 1500]))
        .map_err(|err| SearchError::from(err))
        .and_then(|(_sock, buf, n, _addr)| {
            str::from_utf8(&buf[..n])
                .map_err(|err| SearchError::from(err))
                .and_then(|text| parse_result(text).ok_or(SearchError::InvalidResponse))
        })
        .and_then(move |location| {
            get_control_url(&location, &handle)
                .and_then(move |control_url| Ok(Gateway::new(location.0, control_url, handle)))
        });
    let timeout = Timer::default().timeout(task, timeout);
    Box::new(timeout)
}

pub fn get_control_url(
    location: &(SocketAddrV4, String),
    handle: &Handle,
) -> Box<Future<Item = String, Error = SearchError>> {
    let client = hyper::Client::new(handle);
    let uri = match format!("http://{}{}", location.0, location.1).parse() {
        Ok(uri) => uri,
        Err(err) => return Box::new(future::err(SearchError::from(err))),
    };
    let future = client.get(uri).and_then(|resp| resp.body().concat2()).then(
        |result| match result {
            Ok(body) => parse_control_url(body.as_ref()),
            Err(err) => Err(SearchError::from(err)),
        },
    );
    Box::new(future)
}

fn parse_control_url(resp: &[u8]) -> Result<String, SearchError> {
    let mut parser = Reader::from_reader(resp);
    parser.trim_text(true).expand_empty_elements(false);

    struct Service {
        service_type: Vec<u8>,
        control_url: Vec<u8>,
    }

    let mut service = Service {
        service_type: vec![],
        control_url: vec![],
    };

    #[derive(Clone, Copy, PartialEq)]
    enum Node {
        Device,
        ServiceList,
        Service,
        ServiceType,
        ControlUrl,
        Ignored
    }

    let mut buf = Vec::with_capacity(resp.len());
    let mut chain = [Node::Ignored; 4];

    loop {
        match parser.read_event(&mut buf)? {
            Event::Start(e) => {
                chain.rotate_left(1);
                match e.name() {
                    b"device" => chain[3] = Node::Device,
                    b"serviceList" => chain[3] = Node::ServiceList,
                    b"service" => chain[3] = Node::Service,
                    b"serviceType" => chain[3] = Node::ServiceType,
                    b"controlURL" => chain[3] = Node::ControlUrl,
                    _ => chain[3] = Node::Ignored,
                }
                if &chain[1..] == &[Node::Device, Node::ServiceList, Node::Service] {
                    service.service_type.clear();
                    service.control_url.clear();
                }
            }
            Event::End(_) => {
                if &chain[1..] == &[Node::Device, Node::ServiceList, Node::Service]
                    && (&*service.service_type == b"urn:schemas-upnp-org:service:WANIPConnection:1".as_ref()
                        || &*service.service_type == b"urn:schemas-upnp-org:service:WANPPPConnection:1".as_ref())
                    && !service.control_url.is_empty()
                {
                    return Ok(parser.decode(&service.control_url).into_owned());
                }
            }
            Event::Text(e) => {
                if chain == [Node::Device, Node::ServiceList, Node::Service, Node::ServiceType] {
                    service.service_type.extend_from_slice(&*e.unescaped()?);
                } else if chain == [Node::Device, Node::ServiceList, Node::Service, Node::ControlUrl] {
                    service.control_url.extend_from_slice(&*e.unescaped()?);
                }
            }
            Event::Eof => return Err(SearchError::InvalidResponse),
            _ => (),
        }
        buf.clear();
    }
}
