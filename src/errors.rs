use std::io;
use std::fmt;
use std;

use hyper;

 use soap;

/// Errors that can occur when sending the request to the gateway.
#[derive(Debug)]
pub enum RequestError {
    /// Http/Hyper error
    HttpError(hyper::Error),
    /// IO Error
    IoError(io::Error),
    /// The response from the gateway could not be parsed.
    InvalidResponse(String),
    /// The gateway returned an unhandled error code and description.
    ErrorCode(u16, String),
}

/// Errors returned by `Gateway::get_external_ip`
#[derive(Debug)]
pub enum GetExternalIpError {
    /// The client is not authorized to perform the operation.
    ActionNotAuthorized,
    /// Some other error occured performing the request.
    RequestError(RequestError),
}

/// Errors returned by `Gateway::remove_port`
#[derive(Debug)]
pub enum RemovePortError {
    /// The client is not authorized to perform the operation.
    ActionNotAuthorized,
    /// No such port mapping.
    NoSuchPortMapping,
    /// Some other error occured performing the request.
    RequestError(RequestError),
}

/// Errors returned by `Gateway::add_any_port` and `Gateway::get_any_address`
#[derive(Debug)]
pub enum AddAnyPortError {
    /// The client is not authorized to perform the operation.
    ActionNotAuthorized,
    /// Can not add a mapping for local port 0.
    InternalPortZeroInvalid,
    /// The gateway does not have any free ports.
    NoPortsAvailable,
    /// The gateway can only map internal ports to same-numbered external ports
    /// and this external port is in use.
    ExternalPortInUse,
    /// The gateway only supports permanent leases (ie. a `lease_duration` of 0).
    OnlyPermanentLeasesSupported,
    /// The description was too long for the gateway to handle.
    DescriptionTooLong,
    /// Some other error occured performing the request.
    RequestError(RequestError),
}

impl From<RequestError> for AddAnyPortError {
    fn from(err: RequestError) -> AddAnyPortError {
        AddAnyPortError::RequestError(err)
    }
}

/// Errors returned by `Gateway::add_port`
#[derive(Debug)]
pub enum AddPortError {
    /// The client is not authorized to perform the operation.
    ActionNotAuthorized,
    /// Can not add a mapping for local port 0.
    InternalPortZeroInvalid,
    /// External port number 0 (any port) is considered invalid by the gateway.
    ExternalPortZeroInvalid,
    /// The requested mapping conflicts with a mapping assigned to another client.
    PortInUse,
    /// The gateway requires that the requested internal and external ports are the same.
    SamePortValuesRequired,
    /// The gateway only supports permanent leases (ie. a `lease_duration` of 0).
    OnlyPermanentLeasesSupported,
    /// The description was too long for the gateway to handle.
    DescriptionTooLong,
    /// Some other error occured performing the request.
    RequestError(RequestError),
}

impl From<io::Error> for RequestError {
    fn from(err: io::Error) -> RequestError {
        RequestError::IoError(err)
    }
}

impl From<soap::Error> for RequestError {
    fn from(err: soap::Error) -> RequestError {
        match err {
            soap::Error::HttpError(e) => RequestError::HttpError(e),
            soap::Error::IoError(e) => RequestError::IoError(e),
        }
    }
}

impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RequestError::HttpError(ref e) => write!(f, "HTTP error. {}", e),
            RequestError::InvalidResponse(ref e) => write!(f, "Invalid response from gateway: {}", e),
            RequestError::IoError(ref e) => write!(f, "IO error. {}", e),
            RequestError::ErrorCode(n, ref e) => write!(f, "Gateway response error {}: {}", n, e),
        }
    }
}

impl std::error::Error for RequestError {
    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            RequestError::HttpError(ref e)     => Some(e),
            RequestError::InvalidResponse(..)  => None,
            RequestError::IoError(ref e)       => Some(e),
            RequestError::ErrorCode(..)        => None,
        }
    }

    fn description(&self) -> &str {
        match *self {
            RequestError::HttpError(..)       => "Http error",
            RequestError::InvalidResponse(..) => "Invalid response",
            RequestError::IoError(..)         => "IO error",
            RequestError::ErrorCode(_, ref e) => &e[..],
        }
    }
}

impl fmt::Display for GetExternalIpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            GetExternalIpError::ActionNotAuthorized
                => write!(f, "The client is not authorized to remove the port"),
            GetExternalIpError::RequestError(ref e)
                => write!(f, "Request Error. {}", e),
        }
    }
}

impl From<io::Error> for GetExternalIpError {
    fn from(err: io::Error) -> GetExternalIpError {
        GetExternalIpError::RequestError(RequestError::from(err))
    }
}

impl std::error::Error for GetExternalIpError {
    fn cause(&self) -> Option<&std::error::Error> {
        None
    }

    fn description(&self) -> &str {
        match *self {
            GetExternalIpError::ActionNotAuthorized
                => "The client is not authorized to remove the port",
            GetExternalIpError::RequestError(..)
                => "Request error",
        }
    }
}

impl fmt::Display for RemovePortError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RemovePortError::ActionNotAuthorized
                => write!(f, "The client is not authorized to remove the port"),
            RemovePortError::NoSuchPortMapping
                => write!(f, "The port was not mapped"),
            RemovePortError::RequestError(ref e)
                => write!(f, "Request error. {}", e),
        }
    }
}

impl std::error::Error for RemovePortError {
    fn cause(&self) -> Option<&std::error::Error> {
        None
    }

    fn description(&self) -> &str {
        match *self {
            RemovePortError::ActionNotAuthorized
                => "The client is not authorized to remove the port",
            RemovePortError::NoSuchPortMapping
                => "The port was not mapped",
            RemovePortError::RequestError(..)
                => "Request error",
        }
    }
}

impl fmt::Display for AddAnyPortError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AddAnyPortError::ActionNotAuthorized
                => write!(f, "The client is not authorized to remove the port"),
            AddAnyPortError::InternalPortZeroInvalid
                => write!(f, "Can not add a mapping for local port 0"),
            AddAnyPortError::NoPortsAvailable
                => write!(f, "The gateway does not have any free ports"),
            AddAnyPortError::OnlyPermanentLeasesSupported
                => write!(f, "The gateway only supports permanent leases (ie. a `lease_duration` of 0),"),
            AddAnyPortError::ExternalPortInUse
                => write!(f, "The gateway can only map internal ports to same-numbered external ports and this external port is in use."),
            AddAnyPortError::DescriptionTooLong
                => write!(f, "The description was too long for the gateway to handle."),
            AddAnyPortError::RequestError(ref e)
                => write!(f, "Request error. {}", e),
        }
    }
}

impl std::error::Error for AddAnyPortError {
    fn cause(&self) -> Option<&std::error::Error> {
        None
    }

    fn description(&self) -> &str {
        match *self {
            AddAnyPortError::ActionNotAuthorized
                => "The client is not authorized to remove the port",
            AddAnyPortError::InternalPortZeroInvalid
                => "Can not add a mapping for local port 0.",
            AddAnyPortError::NoPortsAvailable
                => "The gateway does not have any free ports",
            AddAnyPortError::OnlyPermanentLeasesSupported
                => "The gateway only supports permanent leases (ie. a `lease_duration` of 0),",
            AddAnyPortError::ExternalPortInUse
                => "The gateway can only map internal ports to same-numbered external ports and this external port is in use.",
            AddAnyPortError::DescriptionTooLong
                => "The description was too long for the gateway to handle.",
            AddAnyPortError::RequestError(..)
                => "Request error",
        }
    }
}

impl fmt::Display for AddPortError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AddPortError::ActionNotAuthorized
                => write!(f, "The client is not authorized to map this port."),
            AddPortError::InternalPortZeroInvalid
                => write!(f, "Can not add a mapping for local port 0"),
            AddPortError::ExternalPortZeroInvalid
                => write!(f, "External port number 0 (any port) is considered invalid by the gateway."),
            AddPortError::PortInUse
                => write!(f, "The requested mapping conflicts with a mapping assigned to another client."),
            AddPortError::SamePortValuesRequired
                => write!(f, "The gateway requires that the requested internal and external ports are the same."),
            AddPortError::OnlyPermanentLeasesSupported
                => write!(f, "The gateway only supports permanent leases (ie. a `lease_duration` of 0),"),
            AddPortError::DescriptionTooLong
                => write!(f, "The description was too long for the gateway to handle."),
            AddPortError::RequestError(ref e)
                => write!(f, "Request error. {}", e),
        }
    }
}

impl std::error::Error for AddPortError {
    fn cause(&self) -> Option<&std::error::Error> {
        None
    }

    fn description(&self) -> &str {
        match *self {
            AddPortError::ActionNotAuthorized
                => "The client is not authorized to map this port.",
            AddPortError::InternalPortZeroInvalid
                => "Can not add a mapping for local port 0",
            AddPortError::ExternalPortZeroInvalid
                => "External port number 0 (any port) is considered invalid by the gateway.",
            AddPortError::PortInUse
                => "The requested mapping conflicts with a mapping assigned to another client.",
            AddPortError::SamePortValuesRequired
                => "The gateway requires that the requested internal and external ports are the same.",
            AddPortError::OnlyPermanentLeasesSupported
                => "The gateway only supports permanent leases (ie. a `lease_duration` of 0),",
            AddPortError::DescriptionTooLong
                => "The description was too long for the gateway to handle.",
            AddPortError::RequestError(..)
                => "Request error",
        }
    }
}
