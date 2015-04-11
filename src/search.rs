use std::error::Error;
use std::fmt;
use std::io;
use std::net;
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

impl fmt::Display for SearchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SearchError::IoError(ref err) => err.fmt(f),
            SearchError::InvalidResponse => write!(f, "{}", self.description()),
        }
    }
}

impl From<io::Error> for SearchError {
    fn from(err: io::Error) -> SearchError {
        SearchError::IoError(err)
    }
}

impl Error for SearchError {
    fn description(&self) -> &str {
        match *self {
            SearchError::IoError(ref err) => err.description(),
            SearchError::InvalidResponse => "Invalid response received from router",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            SearchError::IoError(ref err) => err.cause(),
            SearchError::InvalidResponse => None,
        }
    }
}

// Try to find the gateway on the local network.
pub fn search_gateway() -> Result<net::SocketAddr, SearchError> {
    let local_ip = try!(get_local_ip());
    let local_addr = (local_ip, 1900);
    let socket = try!(net::UdpSocket::bind(local_addr));
    //socket.set_read_timeout(Some(3000)); // 3 seconds timeout TODO deadline

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

// This is a hacky way of getting this computer's address
// on the local network. There doesn't seem to be another
// way of doing this at the moment.
fn get_local_ip() -> io::Result<net::IpAddr> {
    let sock = try!(net::TcpStream::connect("google.ca:80"));
    match sock.local_addr() {
        Err(err) => Err(err),
        Ok(sockaddr) => Ok(sockaddr.ip())
    }
}

// Parse the result.
fn parse_result(text: &str) -> Option<net::SocketAddr> {
    let re = regex!(r"LOCATION:\s*http://(\d+\.\d+\.\d+\.\d+):(\d+)/.+");
    for line in text.lines() {
        match re.captures(line) {
            None => continue,
            Some(cap) => {
                // these shouldn't fail if the regex matched.
                let addr = cap.at(1).unwrap();
                let port = cap.at(2).unwrap();
                return Some(net::SocketAddr::new(
                    addr.parse::<net::IpAddr>().unwrap(),
                    port.parse::<u16>().unwrap(),
                ))
            },
        }
    }
    None
}
