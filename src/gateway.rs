use std::net::SocketAddrV4;
use std::fmt;

#[derive(Debug)]
pub struct Gateway {
    pub addr: SocketAddrV4,
    pub control_url: String,
}

impl Gateway {
    pub fn new(addr: SocketAddrV4, control_url: String) -> Gateway {
        Gateway{
            addr: addr,
            control_url: control_url
        }
    }
}

impl fmt::Display for Gateway {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "http://{}{}", self.addr, self.control_url)
    }
}
