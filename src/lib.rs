extern crate hyper;
extern crate regex;
extern crate xml;

// data structures
pub use self::gateway::Gateway;
pub use self::gateway::PortMappingProtocol;
pub use self::gateway::RequestError;

// search of gateway
pub use self::search::search_gateway;
pub use self::search::search_gateway_from;
pub use self::search::SearchError;

#[cfg(feature = "unstable")]
pub use self::search::search_gateway_timeout;
#[cfg(feature = "unstable")]
pub use self::search::search_gateway_from_timeout;

// re-export error types
pub use hyper::Error as HttpError;
pub use xml::common::Error as XmlError;

mod gateway;
mod search;
mod soap;
