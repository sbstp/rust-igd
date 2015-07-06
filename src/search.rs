use std::io;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::net::UdpSocket;
use std::str;

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
pub fn search_gateway_from(ip: Ipv4Addr) -> Result<SocketAddrV4, SearchError> {
    let addr = SocketAddrV4::new(ip, 1900);
    let socket = try!(UdpSocket::bind(addr));

    // send the request on the broadcast address
    try!(socket.send_to(SEARCH_REQUEST.as_bytes(), "239.255.255.250:1900"));

    let mut buf = [0u8; 1024];
    let (read, _) = try!(socket.recv_from(&mut buf));
    let text = str::from_utf8(&buf[..read]).unwrap();
    match parse_result(text) {
        None => Err(SearchError::InvalidResponse),
        Some(socketaddr) => Ok(socketaddr),
    }
}

// Try to find the gateway on the local network.
// Bind to all interfaces.
pub fn search_gateway() -> Result<SocketAddrV4, SearchError> {
    search_gateway_from(Ipv4Addr::new(0, 0, 0, 0))
}

// Parse the result.
fn parse_result(text: &str) -> Option<SocketAddrV4> {
    let re = regex!(r"(?i:Location):\s*http://(\d+\.\d+\.\d+\.\d+):(\d+)/.+");
    for line in text.lines() {
        match re.captures(line) {
            None => continue,
            Some(cap) => {
                // these shouldn't fail if the regex matched.
                let addr = cap.at(1).unwrap();
                let port = cap.at(2).unwrap();
                return Some(SocketAddrV4::new(
                    addr.parse::<Ipv4Addr>().unwrap(),
                    port.parse::<u16>().unwrap()
                ));
            },
        }
    }
    None
}
