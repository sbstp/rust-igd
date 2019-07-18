use std::net::{SocketAddrV4, UdpSocket};
use std::str;

use common::{messages, parsing, SearchOptions};
use errors::SearchError;
use gateway::Gateway;

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
            // Defaults to using the first control url.
            if control_url.len() > 0{
                return Ok(Gateway {
                addr: location.0,
                control_url: control_url[0].clone(),
                });
            } else {
                return Err(SearchError::InvalidResponse);
            }
        }
    }
}

pub fn get_control_urls(options: SearchOptions) -> Result<Vec<String>, SearchError> {
    let socket = UdpSocket::bind(options.bind_addr)?;
    socket.set_read_timeout(options.timeout)?;

    socket.send_to(messages::SEARCH_REQUEST.as_bytes(), options.broadcast_address)?;

    loop {
        let mut buf = [0u8; 1500];
        let (read, _) = socket.recv_from(&mut buf)?;
        let text = str::from_utf8(&buf[..read])?;

        let location = parsing::parse_search_result(text)?;

        if let Ok(control_url) = get_control_url(&location) {
            if control_url.len() > 0{
                return Ok(control_url);
            } else {
                return Err(SearchError::InvalidResponse);
            }
        }
    }
}

/*
    A bit of an ugly temporary workaround, basically the idea is to use get_control_urls() and then call
    search_gateway() with a valid control url. Allows the user to select the interface to use.
*/
pub fn get_gateway_with_control_url(options: SearchOptions, url: &str) -> Result<Gateway, SearchError>{
    let mut gateway = search_gateway(options)?;
    gateway.control_url = String::from(url);
    Ok(gateway)
}


fn get_control_url(location: &(SocketAddrV4, String)) -> Result<Vec<String>, SearchError> {
    let url = format!("http://{}:{}{}", location.0.ip(), location.0.port(), location.1);
    let response = attohttpc::get(&url).send()?;
    let res = parsing::parse_control_url(&response.bytes()?[..]);
    res
}

#[test]
fn test_get_control_urls(){
    // This test will fail if upnp is disabled on the default interface ( default gateway )
    assert_eq!(get_control_urls(SearchOptions::default()).unwrap().len() > 0, true);
}