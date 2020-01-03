use std::net::{SocketAddrV4, UdpSocket};
use std::str;

use crate::common::{messages, parsing, SearchOptions};
use crate::errors::SearchError;
use crate::gateway::Gateway;

/// Search gateway, using the given `SearchOptions`.
///
/// The default `SearchOptions` should suffice in most cases.
/// It can be created with `Default::default()` or `SearchOptions::default()`.
///
/// # Example
/// ```no_run
/// use igd::{search_gateway, SearchOptions, Result};
///
/// fn main() -> Result {
///     let gateway = search_gateway(Default::default())?;
///     let ip = gateway.get_external_ip()?;
///     println!("External IP address: {}", ip);
///     Ok(())
/// }
/// ```
pub fn search_gateway(options: SearchOptions) -> Result<Gateway, SearchError> {
    let socket = UdpSocket::bind(options.bind_addr)?;
    socket.set_read_timeout(options.timeout)?;

    socket.send_to(messages::SEARCH_REQUEST.as_bytes(), options.broadcast_address)?;

    loop {
        let mut buf = [0u8; 1500];
        let (read, _) = socket.recv_from(&mut buf)?;
        let text = str::from_utf8(&buf[..read])?;

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
