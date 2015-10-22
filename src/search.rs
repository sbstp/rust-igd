use std::io;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::net::UdpSocket;
use std::str;

#[cfg(feature = "unstable")]
use std::time::Duration;

use hyper;
use regex::Regex;
use xml::EventReader;
use xml::common::Error as XmlError;
use xml::reader::events::XmlEvent;

use gateway::Gateway;

// Content of the request.
const SEARCH_REQUEST: &'static str =
"M-SEARCH * HTTP/1.1\r
Host:239.255.255.250:1900\r
ST:urn:schemas-upnp-org:device:InternetGatewayDevice:1\r
Man:\"ssdp:discover\"\r
MX:3\r\n\r\n";

// Error type this module emits.
#[derive(Debug)]
pub enum SearchError {
    HttpError(hyper::Error),
    IoError(io::Error),
    InvalidResponse,
    Utf8Error(str::Utf8Error),
    XmlError(XmlError),
}

impl From<hyper::Error> for SearchError {
    fn from(err: hyper::Error) -> SearchError {
        SearchError::HttpError(err)
    }
}

impl From<io::Error> for SearchError {
    fn from(err: io::Error) -> SearchError {
        SearchError::IoError(err)
    }
}

impl From<str::Utf8Error> for SearchError {
    fn from(err: str::Utf8Error) -> SearchError {
        SearchError::Utf8Error(err)
    }
}

impl From<XmlError> for SearchError {
    fn from(err: XmlError) -> SearchError {
        SearchError::XmlError(err)
    }
}

fn search_gateway_common(socket: UdpSocket) -> Result<Gateway, SearchError> {
    // send the request on the broadcast address
    try!(socket.send_to(SEARCH_REQUEST.as_bytes(), "239.255.255.250:1900"));
    let mut buf = [0u8; 1024];
    let (read, _) = try!(socket.recv_from(&mut buf));
    let text = try!(str::from_utf8(&buf[..read]));
    match parse_result(text) {
        None => Err(SearchError::InvalidResponse),
        Some(location) => {
            let control_url = try!(get_control_url(&location));
            Ok(Gateway::new(location.0, control_url))
        },
    }
}

// Try to find the gateway on the local network.
// Bind to the given interface. Block for the given duration waiting for a response from the
// gateway.
#[cfg(feature = "unstable")]
pub fn search_gateway_from_timeout(ip: Ipv4Addr, timeout: Duration) -> Result<Gateway, SearchError> {
    let addr = SocketAddrV4::new(ip, 0);
    let socket = try!(UdpSocket::bind(addr));
    try!(socket.set_read_timeout(Some(timeout)));
    search_gateway_common(socket)
}

// Try to find the gateway on the local network.
// Bind to the given interface. Block indefinitely waiting for a response from the gateway.
pub fn search_gateway_from(ip: Ipv4Addr) -> Result<Gateway, SearchError> {
    let addr = SocketAddrV4::new(ip, 0);
    let socket = try!(UdpSocket::bind(addr));
    search_gateway_common(socket)
}

// Try to find the gateway on the local network.
// Bind to all interfaces. Block for the given duration waiting for a response from the gateway.
#[cfg(feature = "unstable")]
pub fn search_gateway_timeout(timeout: Duration) -> Result<Gateway, SearchError> {
    search_gateway_from_timeout(Ipv4Addr::new(0, 0, 0, 0), timeout)
}

// Try to find the gateway on the local network.
// Bind to all interfaces. Block indefinitely waiting for a response from the gateway.
pub fn search_gateway() -> Result<Gateway, SearchError> {
    search_gateway_from(Ipv4Addr::new(0, 0, 0, 0))
}

// Parse the result.
fn parse_result(text: &str) -> Option<(SocketAddrV4, String)> {
    let re = Regex::new(r"(?i:Location):\s*http://(\d+\.\d+\.\d+\.\d+):(\d+)(/[^\r]*)").unwrap();
    for line in text.lines() {
        match re.captures(line) {
            None => continue,
            Some(cap) => {
                // these shouldn't fail if the regex matched.
                let addr = cap.at(1).unwrap();
                let port = cap.at(2).unwrap();
                return Some(
                    (SocketAddrV4::new(
                        addr.parse::<Ipv4Addr>().unwrap(),
                        port.parse::<u16>().unwrap()),
                     cap.at(3).unwrap().to_string()));
            },
        }
    }
    None
}

fn get_control_url(location: &(SocketAddrV4, String)) -> Result<String, SearchError> {
    let client = hyper::Client::new();
    let resp = try!(client.get(&format!("http://{}{}", location.0, location.1)).send());

    let mut parser = EventReader::new(resp);
    let mut chain = Vec::<String>::with_capacity(4);

    struct Service {
        service_type: String,
        control_url: String,
    }

    let mut service = Service{
        service_type: "".to_string(),
        control_url: "".to_string(),
    };

    for e in parser.events() {
        match e {
            XmlEvent::StartElement { name, .. } => {
                chain.push(name.to_repr());
                let tail = if chain.len() >= 3 {
                    chain.iter().skip(chain.len() - 3)
                } else {
                    continue
                };

                if vec!["device", "serviceList", "service"]
                        .iter()
                        .zip(tail)
                        .all(|(l, r)| l == r) {
                    service.service_type.clear();
                    service.control_url.clear();
                }
            },
            XmlEvent::EndElement { .. } => {
                let top = chain.pop();
                let tail = if top == Some("service".to_string())
                        && chain.len() >= 2 {
                    chain.iter().skip(chain.len() - 2)
                } else {
                    continue
                };

                if vec!["device", "serviceList"]
                        .iter()
                        .zip(tail)
                        .all(|(l, r)| l == r) {
                    if "urn:schemas-upnp-org:service:WANIPConnection:1"
                            == service.service_type
                            && service.control_url.len() != 0 {
                        return Ok(service.control_url);
                    }
                }
            },
            XmlEvent::Characters(text) => {
                let tail = if chain.len() >= 4 {
                    chain.iter().skip(chain.len() - 4)
                } else {
                    continue
                };

                if vec!["device", "serviceList", "service", "serviceType"]
                    .iter().zip(tail.clone()).all(|(l, r)| l == r) {
                    service.service_type.push_str(&text);
                }
                if vec!["device", "serviceList", "service", "controlURL"]
                    .iter().zip(tail).all(|(l, r)| l == r) {
                    service.control_url.push_str(&text);
                }
            },
            XmlEvent::Error(e) =>  return Err(e.into()),
            _ => (),
        }
    }
    Err(SearchError::InvalidResponse)
}

#[test]
fn test_parse_result_case_insensitivity() {
    assert!(parse_result("location:http://0.0.0.0:0/control_url").is_some());
    assert!(parse_result("LOCATION:http://0.0.0.0:0/control_url").is_some());
}

#[test]
fn test_parse_result() {
    let result = parse_result("location:http://0.0.0.0:0/control_url").unwrap();
    assert_eq!(result.0.ip(), &Ipv4Addr::new(0,0,0,0));
    assert_eq!(result.0.port(), 0);
    assert_eq!(&result.1[..], "/control_url");
}
