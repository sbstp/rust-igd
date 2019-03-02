use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::str;
use std::time::Duration;

use regex::Regex;

use common::parsing;
use errors::SearchError;
use gateway::Gateway;

// Content of the request.
pub const SEARCH_REQUEST: &'static str = "M-SEARCH * HTTP/1.1\r
Host:239.255.255.250:1900\r
ST:urn:schemas-upnp-org:device:InternetGatewayDevice:1\r
Man:\"ssdp:discover\"\r
MX:3\r\n\r\n";

/// Search gateway, bind to all interfaces and use a timeout of 3 seconds.
///
/// Bind to all interfaces.
/// The request will timeout after 3 seconds.
pub fn search_gateway() -> Result<Gateway, SearchError> {
    search_gateway_timeout(Duration::from_secs(3))
}

/// Search gateway, bind to all interfaces and use the given duration for the timeout.
///
/// Bind to all interfaces.
/// The request will timeout after the given duration.
pub fn search_gateway_timeout(timeout: Duration) -> Result<Gateway, SearchError> {
    search_gateway_from_timeout(Ipv4Addr::new(0, 0, 0, 0), timeout)
}

/// Search gateway, bind to the given interface and use a time of 3 seconds.
///
/// Bind to the given interface.
/// The request will timeout after 3 seconds.
pub fn search_gateway_from(ip: Ipv4Addr) -> Result<Gateway, SearchError> {
    search_gateway_from_timeout(ip, Duration::from_secs(3))
}

/// Search gateway, bind to the given interface and use the given duration for the timeout.
///
/// Bind to the given interface.
/// The request will timeout after the given duration.
pub fn search_gateway_from_timeout(ip: Ipv4Addr, timeout: Duration) -> Result<Gateway, SearchError> {
    let addr = SocketAddrV4::new(ip, 0);
    let socket = try!(UdpSocket::bind(addr));
    try!(socket.set_read_timeout(Some(timeout)));

    try!(socket.send_to(SEARCH_REQUEST.as_bytes(), "239.255.255.250:1900",));
    loop {
        let mut buf = [0u8; 1024];
        let (read, _) = try!(socket.recv_from(&mut buf));
        let text = try!(str::from_utf8(&buf[..read]));

        match parse_result(text) {
            None => return Err(SearchError::InvalidResponse),
            Some(location) => match get_control_url(&location) {
                Ok(control_url) => {
                    return Ok(Gateway {
                        addr: location.0,
                        control_url: control_url,
                    });
                }
                _ => (),
            },
        }
    }
}

// Parse the result.
pub fn parse_result(text: &str) -> Option<(SocketAddrV4, String)> {
    let re = Regex::new(r"(?i:Location):\s*http://(\d+\.\d+\.\d+\.\d+):(\d+)(/[^\r]*)").unwrap();
    for line in text.lines() {
        match re.captures(line) {
            None => continue,
            Some(cap) => {
                // these shouldn't fail if the regex matched.
                let addr = &cap[1];
                let port = &cap[2];
                return Some((
                    SocketAddrV4::new(addr.parse::<Ipv4Addr>().unwrap(), port.parse::<u16>().unwrap()),
                    cap[3].to_string(),
                ));
            }
        }
    }
    None
}

fn get_control_url(location: &(SocketAddrV4, String)) -> Result<String, SearchError> {
    let url = format!("http://{}:{}{}", location.0.ip(), location.0.port(), location.1);
    let (_, _, body) = lynx::Request::get(&url).send()?;
    parsing::parse_control_url(&body.bytes()?[..])
}

#[test]
fn test_parse_result_case_insensitivity() {
    assert!(parse_result("location:http://0.0.0.0:0/control_url").is_some());
    assert!(parse_result("LOCATION:http://0.0.0.0:0/control_url").is_some());
}

#[test]
fn test_parse_result() {
    let result = parse_result("location:http://0.0.0.0:0/control_url").unwrap();
    assert_eq!(result.0.ip(), &Ipv4Addr::new(0, 0, 0, 0));
    assert_eq!(result.0.port(), 0);
    assert_eq!(&result.1[..], "/control_url");
}
