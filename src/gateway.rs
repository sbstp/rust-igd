use std::io;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::fmt;
use std;

use hyper;
use xmltree;

use soap;

/// Errors that can occur when sending the request to the gateway.
#[derive(Debug)]
pub enum RequestError<E: fmt::Debug> {
    /// Http/Hyper error
    HttpError(hyper::Error),
    /// Unable to process the response
    InvalidResponse,
    /// IO Error
    IoError(io::Error),
    /// The gateway responded with an error.
    ErrorResponse(E),
}

#[derive(Debug)]
pub enum GetExternalIpError {
    /// The client is not authorized to perform the operation.
    ActionNotAuthorized,
    /// The gateway returned an unrecognized error string.
    ErrorString(String),
}

#[derive(Debug)]
pub enum RemovePortError {
    /// The client is not authorized to perform the operation.
    ActionNotAuthorized,
    /// No such port mapping.
    NoSuchPortMapping,
    /// The gateway returned an unrecognized error string.
    ErrorString(String),
}

/// Errors returned by the gateway when trying to add a port.
#[derive(Debug)]
pub enum AddPortError {
    /// The client is not authorized to perform the operation.
    ActionNotAuthorized,
    /// External port number 0 (any port) is considered invalid by the gateway.
    WildCardNotPermittedInExtPort,
    /// The requested mapping conflicts with a mapping assigned to another client.
    ConflictInMappingEntry,
    /// This gateway requires that the requested internal and external ports are the same.
    SamePortValuesRequired,
    /// This gateway only supports permanent leases (ie. a `lease_duration` of 0).
    OnlyPermanentLeasesSupported,
    /// The gateway returned an unrecognized error string.
    ErrorString(String),
}

impl<E: fmt::Debug> From<io::Error> for RequestError<E> {
    fn from(err: io::Error) -> RequestError<E> {
        RequestError::IoError(err)
    }
}

impl<E: fmt::Debug> From<soap::Error> for RequestError<E> {
    fn from(err: soap::Error) -> RequestError<E> {
        match err {
            soap::Error::HttpError(e) => RequestError::HttpError(e),
            soap::Error::IoError(e) => RequestError::IoError(e),
        }
    }
}

impl<E: fmt::Debug + fmt::Display> fmt::Display for RequestError<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RequestError::HttpError(ref e) => write!(f, "Http error. {}", e),
            RequestError::InvalidResponse => write!(f, "Invalid response from gateway."),
            RequestError::IoError(ref e) => write!(f, "IO error. {}", e),
            RequestError::ErrorResponse(ref e) => write!(f, "The gateway responded with an error. {}", e),
        }
    }
}

impl<E: std::error::Error> std::error::Error for RequestError<E> {
    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            RequestError::HttpError(ref e)     => Some(e),
            RequestError::InvalidResponse      => None,
            RequestError::IoError(ref e)       => Some(e),
            RequestError::ErrorResponse(ref e) => Some(e),
        }
    }

    fn description(&self) -> &str {
        match *self {
            RequestError::HttpError(..)     => "Http error",
            RequestError::InvalidResponse   => "Invalid response",
            RequestError::IoError(..)       => "IO error",
            RequestError::ErrorResponse(..) => "Error response",
        }
    }
}

impl fmt::Display for GetExternalIpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            GetExternalIpError::ActionNotAuthorized
                => write!(f, "The client is not authorized to remove the port"),
            GetExternalIpError::ErrorString(ref s)
                => write!(f, "The gateway returned an unrecognized error string: \"{}\"", s),
        }
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
            GetExternalIpError::ErrorString(ref s)
                => &s[..],
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
            RemovePortError::ErrorString(ref s)
                => write!(f, "The gateway returned an unrecognized error string: \"{}\"", s),
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
            RemovePortError::ErrorString(ref s)
                => &s[..],
        }
    }
}

impl fmt::Display for AddPortError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AddPortError::ActionNotAuthorized
                => write!(f, "The client is not authorized to map this port."),
            AddPortError::WildCardNotPermittedInExtPort
                => write!(f, "External port number 0 (any port) is considered invalid by the gateway."),
            AddPortError::ConflictInMappingEntry
                => write!(f, "The requested mapping conflicts with a mapping assigned to another client."),
            AddPortError::SamePortValuesRequired
                => write!(f, "This gateway requires that the requested internal and external ports are the same."),
            AddPortError::OnlyPermanentLeasesSupported
                => write!(f, "This gateway only supports permanent leases (ie. a `lease_duration` of 0),"),
            AddPortError::ErrorString(ref s)
                => write!(f, "The gateway returned an unrecognized error string: \"{}\"", s),
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
            AddPortError::WildCardNotPermittedInExtPort
                => "External port number 0 (any port) is considered invalid by the gateway.",
            AddPortError::ConflictInMappingEntry
                => "The requested mapping conflicts with a mapping assigned to another client.",
            AddPortError::SamePortValuesRequired
                => "This gateway requires that the requested internal and external ports are the same.",
            AddPortError::OnlyPermanentLeasesSupported
                => "This gateway only supports permanent leases (ie. a `lease_duration` of 0),",
            AddPortError::ErrorString(ref s)
                => &s[..],
        }
    }
}

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

/// This structure represents a gateway found by the search functions.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Gateway {
    /// Socket address of the gateway
    pub addr: SocketAddrV4,
    /// Control url of the device
    pub control_url: String,
}

impl Gateway {

    /// Get the external IP address of the gateway.
    pub fn get_external_ip(&self) -> Result<Ipv4Addr, RequestError<GetExternalIpError>> {
        use RequestError::*;
        let url = format!("{}", self);
        let body = "<?xml version=\"1.0\"?>
        <SOAP-ENV:Envelope SOAP-ENV:encodingStyle=\"http://schemas.xmlsoap.org/soap/encoding/\" xmlns:SOAP-ENV=\"http://schemas.xmlsoap.org/soap/envelope/\">
            <SOAP-ENV:Body>
                <m:GetExternalIPAddress xmlns:m=\"urn:schemas-upnp-org:service:WANIPConnection:1\">
                </m:GetExternalIPAddress>
            </SOAP-ENV:Body>
        </SOAP-ENV:Envelope>";

        let text = try!(soap::send(&url, soap::Action::new(GET_EXTERNAL_IP_ACTION), body));

        let xml = match xmltree::Element::parse(text.as_bytes()) {
            Ok(xml) => xml,
            Err(..) => return Err(InvalidResponse),
        };

        let body = match xml.get_child("Body")
        {
            Some(body) => body,
            None => return Err(InvalidResponse),
        };
        if let Some(ext_ip) = body.get_child("GetExternalIPAddressResponse")
                                  .and_then(|e| e.get_child("NewExternalIPAddress"))
        {
            match ext_ip.text {
                Some(ref t) => match t.parse::<Ipv4Addr>() {
                    Ok(ipv4_addr) => return Ok(ipv4_addr),
                    Err(..) => return Err(InvalidResponse),
                },
                None => return Err(InvalidResponse),
            }
        };
        if let Some(fault) = body.get_child("Fault") {
            match fault.get_child("detail")
                       .and_then(|e| e.get_child("UPnPError"))
                       .and_then(|e| e.get_child("errorDescription"))
                       .and_then(|e| e.text.as_ref())
            {
                Some(description) => match &description[..] {
                    "Action not authorized" => return Err(ErrorResponse(GetExternalIpError::ActionNotAuthorized)),
                    d => return Err(ErrorResponse(GetExternalIpError::ErrorString(From::from(d)))),
                },
                None => return Err(InvalidResponse),
            };
        }
        Err(InvalidResponse)
    }

    /// Add a port mapping.
    ///
    /// The local_addr is the address where the traffic is sent to.
    /// The lease_duration parameter is in seconds. A value of 0 is infinite.
    pub fn add_port(&self, protocol: PortMappingProtocol,
                    external_port: u16, local_addr: SocketAddrV4, lease_duration: u32,
                    description: &str) -> Result<(), RequestError<AddPortError>> {
        use RequestError::*;
        let url = format!("{}", self);
        let body = format!("<?xml version=\"1.0\"?>
        <s:Envelope xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/\" s:encodingStyle=\"http://schemas.xmlsoap.org/soap/encoding/\">
        <s:Body>
            <u:AddPortMapping xmlns:u=\"urn:schemas-upnp-org:service:WANIPConnection:1\">
                <NewProtocol>{}</NewProtocol>
                <NewExternalPort>{}</NewExternalPort>
                <NewInternalClient>{}</NewInternalClient>
                <NewInternalPort>{}</NewInternalPort>
                <NewLeaseDuration>{}</NewLeaseDuration>
                <NewPortMappingDescription>{}</NewPortMappingDescription>
                <NewEnabled>1</NewEnabled>
                <NewRemoteHost></NewRemoteHost>
            </u:AddPortMapping>
        </s:Body>
        </s:Envelope>
        ", protocol, external_port, local_addr.ip(),
           local_addr.port(), lease_duration, description);

        let text = try!(soap::send(&url, soap::Action::new(ADD_PORT_ACTION), &body));

        let xml = match xmltree::Element::parse(text.as_bytes()) {
            Ok(xml) => xml,
            Err(..) => return Err(InvalidResponse),
        };

        let body = match xml.get_child("Body")
        {
            Some(body) => body,
            None => return Err(InvalidResponse),
        };
        if let Some(..) = body.get_child("AddPortMappingResponse") {
            return Ok(());
        };
        if let Some(fault) = body.get_child("Fault") {
            match fault.get_child("detail")
                       .and_then(|e| e.get_child("UPnPError"))
                       .and_then(|e| e.get_child("errorDescription"))
                       .and_then(|e| e.text.as_ref())
            {
                Some(description) => match &description[..] {
                    "Action not authorized" => return Err(ErrorResponse(AddPortError::ActionNotAuthorized)),
                    "WildCardNotPermittedInExtPort" => return Err(ErrorResponse(AddPortError::WildCardNotPermittedInExtPort)),
                    "ConflictInMappingEntry" => return Err(ErrorResponse(AddPortError::ConflictInMappingEntry)),
                    "SamePortValuesRequired" => return Err(ErrorResponse(AddPortError::SamePortValuesRequired)),
                    "OnlyPermanentLeasesSupported" => return Err(ErrorResponse(AddPortError::OnlyPermanentLeasesSupported)),
                    d => return Err(ErrorResponse(AddPortError::ErrorString(From::from(d)))),
                },
                None => return Err(InvalidResponse),
            };
        }
        Err(InvalidResponse)
    }

    /// Remove a port mapping.
    pub fn remove_port(&self, protocol: PortMappingProtocol,
                       external_port: u16) -> Result<(), RequestError<RemovePortError>> {
        use RequestError::*;
        let url = format!("{}", self);
        let body = format!("<?xml version=\"1.0\"?>
        <s:Envelope xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/\" s:encodingStyle=\"http://schemas.xmlsoap.org/soap/encoding/\">
        <s:Body>
            <u:DeletePortMapping xmlns:u=\"urn:schemas-upnp-org:service:WANIPConnection:1\">
                <NewProtocol>{}</NewProtocol>
                <NewExternalPort>{}</NewExternalPort>
                <NewRemoteHost>
                </NewRemoteHost>
            </u:DeletePortMapping>
        </s:Body>
        </s:Envelope>
        ", protocol, external_port);

        let text = try!(soap::send(&url, soap::Action::new(DELETE_PORT_ACTION), &body));

        let xml = match xmltree::Element::parse(text.as_bytes()) {
            Ok(xml) => xml,
            Err(..) => return Err(InvalidResponse),
        };

        let body = match xml.get_child("Body")
        {
            Some(body) => body,
            None => return Err(InvalidResponse),
        };
        if let Some(..) = body.get_child("DeletePortMappingResponse") {
            return Ok(());
        };
        if let Some(fault) = body.get_child("Fault") {
            match fault.get_child("detail")
                       .and_then(|e| e.get_child("UPnPError"))
                       .and_then(|e| e.get_child("errorDescription"))
                       .and_then(|e| e.text.as_ref())
            {
                Some(description) => match &description[..] {
                    "Action not authorized" => return Err(ErrorResponse(RemovePortError::ActionNotAuthorized)),
                    "NoSuchEntryInArray" => return Err(ErrorResponse(RemovePortError::NoSuchPortMapping)),
                    d => return Err(ErrorResponse(RemovePortError::ErrorString(From::from(d)))),
                },
                None => return Err(InvalidResponse),
            };
        }
        Err(InvalidResponse)
    }
}

impl fmt::Display for Gateway {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "http://{}{}", self.addr, self.control_url)
    }
}


// Content of the get external ip SOAPAction request header.
const GET_EXTERNAL_IP_ACTION: &'static str = "\"urn:schemas-upnp-org:service:WANIPConnection:1#GetExternalIPAddress\"";

// Content of the add port mapping SOAPAction request header.
const ADD_PORT_ACTION: &'static str = "\"urn:schemas-upnp-org:service:WANIPConnection:1#AddPortMapping\"";

// Content of the delete port mapping SOAPAction request header.
const DELETE_PORT_ACTION: &'static str = "\"urn:schemas-upnp-org:service:WANIPConnection:1#DeletePortMapping\"";

