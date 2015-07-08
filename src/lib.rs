#![feature(plugin)]
#![feature(ip_addr)]

extern crate hyper;
extern crate regex;
extern crate xml;

// data structures
pub use self::gateway::Gateway;

// request external ip address
pub use self::external::get_external_ip;
pub use self::external::RequestError;

// request port mapping
pub use self::external::PortMappingProtocol;
pub use self::external::add_port;
pub use self::external::remove_port;

// search of gateway
pub use self::search::search_gateway;
pub use self::search::SearchError;

// re-export error types
pub use hyper::Error as HttpError;
pub use xml::common::Error as XmlError;

mod gateway;
mod external;
mod search;
mod soap;
