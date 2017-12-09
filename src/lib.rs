//! This library allows you to communicate with an IGD enabled device.
//! Use one of the `search_gateway` functions to obtain a `Gateway` object.
//! You can then communicate with the device via this object.

#![deny(missing_docs)]

extern crate hyper;
extern crate regex;
extern crate xml;
extern crate xmltree;
extern crate rand;
extern crate futures;
extern crate tokio_core;
extern crate tokio_timer;
extern crate tokio_retry;

// data structures
pub use self::gateway::Gateway;
pub use self::errors::{RequestError,
                       GetExternalIpError,
                       AddPortError,
                       AddAnyPortError,
                       RemovePortError};

// search of gateway
pub use self::search::search_gateway;
pub use self::search::search_gateway_timeout;
pub use self::search::search_gateway_from;
pub use self::search::search_gateway_from_timeout;
pub use self::search::SearchError;

// re-export error types
pub use hyper::Error as HttpError;
pub use xml::reader::Error as XmlError;

mod gateway;
mod search;
mod soap;
mod async;
mod errors;

use std::fmt;

/// Represents the protocols available for port mapping.
#[derive(Debug,Clone,Copy,PartialEq)]
pub enum PortMappingProtocol {
    /// TCP protocol
    TCP,
    /// UDP protocol
    UDP,
}

impl fmt::Display for PortMappingProtocol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
         write!(f, "{}", match *self {
            PortMappingProtocol::TCP => "TCP",
            PortMappingProtocol::UDP => "UDP",
        })
    }
}
