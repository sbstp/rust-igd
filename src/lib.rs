#![deny(missing_docs)]

//! This library allows you to communicate with an IGD enabled device.
//! Use one of the `search_gateway` functions to obtain a `Gateway` object.
//! You can then communicate with the device via this object.

extern crate attohttpc;
#[macro_use]
extern crate log;
extern crate bytes;

extern crate rand;
extern crate url;
extern crate xmltree;

#[cfg(feature = "async")]
extern crate http;
#[cfg(feature = "async")]
extern crate futures;
#[cfg(feature = "async")]
extern crate hyper;
#[cfg(feature = "async")]
extern crate tokio;
#[cfg(feature = "async")]
extern crate tokio_retry;

// data structures
pub use self::errors::{AddAnyPortError, AddPortError, GetExternalIpError, RemovePortError, RequestError, SearchError};
pub use self::gateway::Gateway;

// search of gateway
pub use self::search::search_gateway;
pub use self::search::search_gateway_from;
pub use self::search::search_gateway_from_timeout;
pub use self::search::search_gateway_timeout;

#[cfg(feature = "async")]
pub mod async;
mod common;
mod errors;
mod gateway;
mod search;

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
