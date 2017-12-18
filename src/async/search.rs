use std::io;
use std::net::{SocketAddr, Ipv4Addr, SocketAddrV4};
use std::str;
use std::time::Duration;

use futures::{Future, IntoFuture, Stream};
use futures::future;
use tokio_core::reactor::Handle;
use tokio_core::net::UdpSocket;
use tokio_timer::Timer;
use hyper;
use xml::EventReader;
use xml::reader::XmlEvent;

use async::Gateway;
use errors::SearchError;
use search::{SEARCH_REQUEST, parse_result};

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
                .and_then(|text| {
                    parse_result(text).ok_or(SearchError::InvalidResponse)
                })
        })
        .and_then(move |location| {
            get_control_url(&location, &handle).and_then(move |control_url| {
                Ok(Gateway::new(location.0, control_url, handle))
            })
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
    let future = client
        .get(uri)
        .and_then(|resp| resp.body().concat2())
        .then(|result| match result {
            Ok(body) => parse_control_url(body.as_ref()),
            Err(err) => Err(SearchError::from(err)),
        });
    Box::new(future)
}

fn parse_control_url<R>(resp: R) -> Result<String, SearchError>
where
    R: io::Read,
{

    let parser = EventReader::new(resp);
    let mut chain = Vec::<String>::with_capacity(4);

    struct Service {
        service_type: String,
        control_url: String,
    }

    let mut service = Service {
        service_type: "".to_string(),
        control_url: "".to_string(),
    };

    for e in parser.into_iter() {
        match try!(e) {
            XmlEvent::StartElement { name, .. } => {
                chain.push(name.borrow().to_repr());
                let tail = if chain.len() >= 3 {
                    chain.iter().skip(chain.len() - 3)
                } else {
                    continue;
                };

                if vec!["device", "serviceList", "service"]
                    .iter()
                    .zip(tail)
                    .all(|(l, r)| l == r)
                {
                    service.service_type.clear();
                    service.control_url.clear();
                }
            }
            XmlEvent::EndElement { .. } => {
                let top = chain.pop();
                let tail = if top == Some("service".to_string()) && chain.len() >= 2 {
                    chain.iter().skip(chain.len() - 2)
                } else {
                    continue;
                };

                if vec!["device", "serviceList"].iter().zip(tail).all(
                    |(l, r)| {
                        l == r
                    },
                )
                {
                    if ("urn:schemas-upnp-org:service:WANIPConnection:1" == service.service_type ||
                            "urn:schemas-upnp-org:service:WANPPPConnection:1" ==
                                service.service_type) &&
                        service.control_url.len() != 0
                    {
                        return Ok(service.control_url);
                    }
                }
            }
            XmlEvent::Characters(text) => {
                let tail = if chain.len() >= 4 {
                    chain.iter().skip(chain.len() - 4)
                } else {
                    continue;
                };

                if vec!["device", "serviceList", "service", "serviceType"]
                    .iter()
                    .zip(tail.clone())
                    .all(|(l, r)| l == r)
                {
                    service.service_type.push_str(&text);
                }
                if vec!["device", "serviceList", "service", "controlURL"]
                    .iter()
                    .zip(tail)
                    .all(|(l, r)| l == r)
                {
                    service.control_url.push_str(&text);
                }
            }
            _ => (),
        }
    }
    Err(SearchError::InvalidResponse)
}
