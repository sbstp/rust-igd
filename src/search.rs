use std::io;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::net::UdpSocket;
use std::str;

use curl::http;

use regex::Regex;

use xml::EventReader;
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
    IoError(io::Error),
    InvalidResponse,
}

impl From<io::Error> for SearchError {
    fn from(err: io::Error) -> SearchError {
        SearchError::IoError(err)
    }
}

// Try to find the gateway on the local network.
// Bind to the given interface.
pub fn search_gateway_from(ip: Ipv4Addr) -> Result<Gateway, SearchError> {
    let addr = SocketAddrV4::new(ip, 0);
    let socket = try!(UdpSocket::bind(addr));

    // send the request on the broadcast address
    try!(socket.send_to(SEARCH_REQUEST.as_bytes(), "239.255.255.250:1900"));
    let mut buf = [0u8; 1024];
    let (read, _) = try!(socket.recv_from(&mut buf));
    let text = str::from_utf8(&buf[..read]).unwrap();
    match parse_result(text) {
        None => Err(SearchError::InvalidResponse),
        Some(location) => {
            Ok(Gateway::new(location.0, match get_control_url(&location) {
                Some(u) => u,
                None => return Err(SearchError::InvalidResponse),
            }))
        },
    }
}

// Try to find the gateway on the local network.
// Bind to all interfaces.
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

fn get_control_url(location: &(SocketAddrV4, String))
                   -> Option<String> {
    let resp = match http::handle()
        .get(format!("http://{}{}", location.0, location.1))
        .exec() {
            Ok(r) => r,
            Err(_) => return None,
        };
    let text = match str::from_utf8(resp.get_body()) {
        Ok(t) => t,
        Err(_) => return None,
    };

    let text = io::Cursor::new(text.as_bytes());
    let mut parser = EventReader::new(text);
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
                        return Some(service.control_url);
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
            XmlEvent::Error(_) =>  return None,
            _ => (),
        }
    }
    None
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
