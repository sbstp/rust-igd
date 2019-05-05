use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::str;
use std::time::Duration;

use common::{messages, parsing};
use errors::SearchError;
use gateway::Gateway;

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

    try!(socket.send_to(messages::SEARCH_REQUEST.as_bytes(), "239.255.255.250:1900",));
    loop {
        let mut buf = [0u8; 1024];
        let (read, _) = try!(socket.recv_from(&mut buf));
        let text = try!(str::from_utf8(&buf[..read]));

        let location = parsing::parse_search_result(text)?;
        if let Ok(control_url) = get_control_url(&location) {
            return Ok(Gateway {
                addr: location.0,
                control_url: control_url,
            });
        }
    }
}

fn get_control_url(location: &(SocketAddrV4, String)) -> Result<String, SearchError> {
    let url = format!("http://{}:{}{}", location.0.ip(), location.0.port(), location.1);
    let response = attohttpc::get(&url).send()?;
    parsing::parse_control_url(&response.bytes()?[..])
}
