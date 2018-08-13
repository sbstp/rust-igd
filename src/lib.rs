//! This library allows you to communicate with an IGD enabled device.
//! Use one of the `search_gateway` functions to obtain a `Gateway` object.
//! You can then communicate with the device via this object.

#![deny(missing_docs)]

extern crate failure;
extern crate futures;
extern crate hyper;
extern crate quick_xml;
extern crate rand;
extern crate regex;
extern crate tokio_core;
extern crate tokio_retry;
extern crate tokio_timer;

// data structures
pub use self::errors::{
    AddAnyPortError, AddPortError, GetExternalIpError, RemovePortError, RequestError, SearchError,
};
pub use self::gateway::Gateway;

// search of gateway
pub use self::search::search_gateway;
pub use self::search::search_gateway_from;
pub use self::search::search_gateway_from_timeout;
pub use self::search::search_gateway_timeout;

/// Contains Tokio compatible implementations for finding a gateway and configuring port mappings
pub mod tokio {
    pub use async::{
        search_gateway, search_gateway_from, search_gateway_from_timeout, search_gateway_timeout,
        Gateway,
    };
}

// re-export error types
pub use hyper::Error as HttpError;
pub use quick_xml::Error as XmlError;

mod async;
mod errors;
mod gateway;
mod search;
mod soap;

use std::fmt;

/// Represents the protocols available for port mapping.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PortMappingProtocol {
    /// TCP protocol
    TCP,
    /// UDP protocol
    UDP,
}

impl fmt::Display for PortMappingProtocol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                PortMappingProtocol::TCP => "TCP",
                PortMappingProtocol::UDP => "UDP",
            }
        )
    }
}
