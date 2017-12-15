use std::net::{Ipv4Addr, SocketAddrV4};
use std::hash::{Hash, Hasher};
use std::fmt;
use rand::distributions::IndependentSample;

use xmltree;
use futures::Future;
use futures::future;
use tokio_core::reactor::Handle;
use tokio_retry::{RetryIf, Error as RetryError};
use tokio_retry::strategy::FixedInterval;
use rand;
use soap;
use errors::{RequestError, GetExternalIpError, AddPortError, AddAnyPortError, RemovePortError};

use PortMappingProtocol;

/// This structure represents a gateway found by the search functions.
#[derive(Clone, Debug)]
pub struct Gateway {
    /// Socket address of the gateway
    addr: SocketAddrV4,
    /// Control url of the device
    control_url: String,

    handle: Handle,
}

impl Gateway {
    /// Create a new Gateway for a given Handle to a control loop
    pub fn new(addr: SocketAddrV4, control_url: String, handle: Handle) -> Gateway {
        Gateway {
            addr: addr,
            control_url: control_url,
            handle: handle,
        }
    }

    fn perform_request(
        &self,
        header: &str,
        body: &str,
        ok: &str,
    ) -> Box<Future<Item = (String, xmltree::Element), Error = RequestError>> {
        let url = format!("{}", self);
        let ok = ok.to_owned();
        let future = soap::send_async(&url, soap::Action::new(header), body, &self.handle)
            .map_err(|err| RequestError::from(err))
            .and_then(move |text| parse_response(text, &ok));
        Box::new(future)
    }

    /// Get the external IP address of the gateway in a tokio compatible way
    pub fn get_external_ip<F>(&self) -> Box<Future<Item = Ipv4Addr, Error = GetExternalIpError>> {
        let header = "\"urn:schemas-upnp-org:service:WANIPConnection:1#GetExternalIPAddress\"";
        let body = "<?xml version=\"1.0\"?>
        <SOAP-ENV:Envelope SOAP-ENV:encodingStyle=\"http://schemas.xmlsoap.org/soap/encoding/\" xmlns:SOAP-ENV=\"http://schemas.xmlsoap.org/soap/envelope/\">
            <SOAP-ENV:Body>
                <m:GetExternalIPAddress xmlns:m=\"urn:schemas-upnp-org:service:WANIPConnection:1\">
                </m:GetExternalIPAddress>
            </SOAP-ENV:Body>
        </SOAP-ENV:Envelope>";
        let future = self.perform_request(header, body, "GetExternalIPAddressResponse")
            .then(|result| match result {
                Ok((text, response)) => {
                    match response
                        .get_child("NewExternalIPAddress")
                        .and_then(|e| e.text.as_ref())
                        .and_then(|t| t.parse::<Ipv4Addr>().ok()) {
                        Some(ipv4_addr) => Ok(ipv4_addr),
                        None => Err(GetExternalIpError::RequestError(
                            RequestError::InvalidResponse(text),
                        )),
                    }
                }
                Err(RequestError::ErrorCode(606, _)) => Err(
                    GetExternalIpError::ActionNotAuthorized,
                ),
                Err(e) => Err(GetExternalIpError::RequestError(e)),
            });
        Box::new(future)
    }

    /// Get an external socket address with our external ip and any port. This is a convenience
    /// function that calls `get_external_ip` followed by `add_any_port`
    ///
    /// The local_addr is the address where the traffic is sent to.
    /// The lease_duration parameter is in seconds. A value of 0 is infinite.
    ///
    /// # Returns
    ///
    /// The external address that was mapped on success. Otherwise an error.
    pub fn get_any_address(
        &self,
        protocol: PortMappingProtocol,
        local_addr: SocketAddrV4,
        lease_duration: u32,
        description: &str,
    ) -> Box<Future<Item = SocketAddrV4, Error = AddAnyPortError>> {
        let description = description.to_owned();
        let gateway = self.clone();
        let future =
            self.get_external_ip::<Box<Future<Item = Ipv4Addr, Error = GetExternalIpError>>>()
                .map_err(|err| match err {
                    GetExternalIpError::ActionNotAuthorized => AddAnyPortError::ActionNotAuthorized,
                    GetExternalIpError::RequestError(e) => AddAnyPortError::RequestError(e),
                })
                .and_then(move |ip| {
                    gateway
                        .add_any_port(protocol, local_addr, lease_duration, &description)
                        .and_then(move |port| Ok(SocketAddrV4::new(ip, port)))
                });
        Box::new(future)

    }


    /// Add a port mapping.with any external port.
    ///
    /// The local_addr is the address where the traffic is sent to.
    /// The lease_duration parameter is in seconds. A value of 0 is infinite.
    ///
    /// # Returns
    ///
    /// The external port that was mapped on success. Otherwise an error.
    pub fn add_any_port(
        &self,
        protocol: PortMappingProtocol,
        local_addr: SocketAddrV4,
        lease_duration: u32,
        description: &str,
    ) -> Box<Future<Item = u16, Error = AddAnyPortError>> {
        // This function first attempts to call AddAnyPortMapping on the IGD with a random port
        // number. If that fails due to the method being unknown it attempts to call AddPortMapping
        // instead with a random port number. If that fails due to ConflictInMappingEntry it retrys
        // with another port up to a maximum of 20 times. If it fails due to SamePortValuesRequired
        // it retrys once with the same port values.

        if local_addr.port() == 0 {
            return Box::new(future::err(AddAnyPortError::InternalPortZeroInvalid));
        }

        let port_range = rand::distributions::Range::new(32768u16, 65535u16);
        let mut rng = rand::thread_rng();
        let external_port = port_range.ind_sample(&mut rng);

        let header = "\"urn:schemas-upnp-org:service:WANIPConnection:1#AddAnyPortMapping\"";
        let body = format!(
            "<?xml version=\"1.0\"?>
        <s:Envelope xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/\" s:encodingStyle=\"http://schemas.xmlsoap.org/soap/encoding/\">
        <s:Body>
            <u:AddAnyPortMapping xmlns:u=\"urn:schemas-upnp-org:service:WANIPConnection:1\">
                <NewProtocol>{}</NewProtocol>
                <NewExternalPort>{}</NewExternalPort>
                <NewInternalClient>{}</NewInternalClient>
                <NewInternalPort>{}</NewInternalPort>
                <NewLeaseDuration>{}</NewLeaseDuration>
                <NewPortMappingDescription>{}</NewPortMappingDescription>
                <NewEnabled>1</NewEnabled>
                <NewRemoteHost></NewRemoteHost>
            </u:AddAnyPortMapping>
        </s:Body>
        </s:Envelope>
        ",
            protocol,
            external_port,
            local_addr.ip(),
            local_addr.port(),
            lease_duration,
            description
        );
        let gateway = self.clone();
        let description = description.to_owned();
        // First, attempt to call the AddAnyPortMapping method.
        let future = self.perform_request(header, &*body, "AddAnyPortMappingResponse")
            .and_then(|(text, response)| {
                        match response.get_child("NewReservedPort")
                            .and_then(|e| e.text.as_ref())
                            .and_then(|t| t.parse::<u16>().ok())
                        {
                            Some(port) => Ok(port),
                            None => Err(RequestError::InvalidResponse(text)),
                        }
            }).or_else(move |err| {
                match err {
                    // The router doesn't know the AddAnyPortMapping method. Try using AddPortMapping
                    // instead.
                    RequestError::ErrorCode(401, _) => {
                        // Try a bunch of random ports.
                        gateway.retry_add_random_port_mapping(protocol, local_addr, lease_duration, &description)
                    },
                    e => {
                        let err = match e {
                            RequestError::ErrorCode(605, _) => AddAnyPortError::DescriptionTooLong,
                            RequestError::ErrorCode(606, _) => AddAnyPortError::ActionNotAuthorized,
                            RequestError::ErrorCode(728, _) => AddAnyPortError::NoPortsAvailable,
                            e => AddAnyPortError::RequestError(e),
                        };
                        Box::new(future::err(err))
                    }
                }
            });
        Box::new(future)
    }

    fn retry_add_random_port_mapping(
        &self,
        protocol: PortMappingProtocol,
        local_addr: SocketAddrV4,
        lease_duration: u32,
        description: &str,
    ) -> Box<Future<Item = u16, Error = AddAnyPortError>> {
        let description = description.to_owned();
        let gateway = self.clone();
        let retry_strategy = FixedInterval::from_millis(0).take(20);
        let future = RetryIf::spawn(
            gateway.handle.clone(),
            retry_strategy,
            move || {
                gateway.add_random_port_mapping(protocol, local_addr, lease_duration, &description)
            },
            |err: &AddAnyPortError| match err {
                &AddAnyPortError::NoPortsAvailable => true,
                _ => false,
            },
        ).map_err(|err| match err {
            RetryError::OperationError(e) => e,
            RetryError::TimerError(io_error) => AddAnyPortError::from(RequestError::from(io_error)),
        });
        Box::new(future)
    }

    fn add_random_port_mapping(
        &self,
        protocol: PortMappingProtocol,
        local_addr: SocketAddrV4,
        lease_duration: u32,
        description: &str,
    ) -> Box<Future<Item = u16, Error = AddAnyPortError>> {
        let description = description.to_owned();
        let gateway = self.clone();
        let port_range = rand::distributions::Range::new(32768u16, 65535u16);
        let mut rng = rand::thread_rng();
        let external_port = port_range.ind_sample(&mut rng);
        let future = self.add_port_mapping(protocol, external_port, local_addr, lease_duration, &description)
            .map(move |_| external_port)
            .or_else(move |err|
                     match err {
                         RequestError::ErrorCode(724, _) =>
                         // The router requires that internal and external ports are the same.
                             gateway.add_same_port_mapping(protocol, local_addr, lease_duration, &description),
                         e => { 
                             let err = match e {
                                 RequestError::ErrorCode(605, _) => AddAnyPortError::DescriptionTooLong,
                                 RequestError::ErrorCode(606, _) => AddAnyPortError::ActionNotAuthorized,
                                 // That port is in use. Try another.
                                 RequestError::ErrorCode(718, _) => AddAnyPortError::NoPortsAvailable,
                                 RequestError::ErrorCode(725, _) => AddAnyPortError::OnlyPermanentLeasesSupported,
                                 e => AddAnyPortError::RequestError(e),
                             };
                             Box::new(future::err(err))
                         }
                     }
            );
        Box::new(future)
    }

    fn add_same_port_mapping(
        &self,
        protocol: PortMappingProtocol,
        local_addr: SocketAddrV4,
        lease_duration: u32,
        description: &str,
    ) -> Box<Future<Item = u16, Error = AddAnyPortError>> {
        let future = self.add_port_mapping(
            protocol,
            local_addr.port(),
            local_addr,
            lease_duration,
            description,
        ).then(move |result| match result {
                Ok(()) => Ok(local_addr.port()),
                Err(RequestError::ErrorCode(606, _)) => Err(AddAnyPortError::ActionNotAuthorized),
                Err(RequestError::ErrorCode(718, _)) => Err(AddAnyPortError::ExternalPortInUse),
                Err(RequestError::ErrorCode(725, _)) => Err(
                    AddAnyPortError::OnlyPermanentLeasesSupported,
                ),
                Err(e) => Err(AddAnyPortError::RequestError(e)),
            });
        Box::new(future)
    }

    fn add_port_mapping(
        &self,
        protocol: PortMappingProtocol,
        external_port: u16,
        local_addr: SocketAddrV4,
        lease_duration: u32,
        description: &str,
    ) -> Box<Future<Item = (), Error = RequestError>> {

        let header = "\"urn:schemas-upnp-org:service:WANIPConnection:1#AddPortMapping\"";
        let body = format!(
            "<?xml version=\"1.0\"?>
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
        ",
            protocol,
            external_port,
            local_addr.ip(),
            local_addr.port(),
            lease_duration,
            description
        );
        let future = self.perform_request(header, &*body, "AddPortMappingResponse")
            .map(|_| ());
        Box::new(future)
    }

    /// Add a port mapping.
    ///
    /// The local_addr is the address where the traffic is sent to.
    /// The lease_duration parameter is in seconds. A value of 0 is infinite.
    pub fn add_port(
        &self,
        protocol: PortMappingProtocol,
        external_port: u16,
        local_addr: SocketAddrV4,
        lease_duration: u32,
        description: &str,
    ) -> Box<Future<Item = (), Error = AddPortError>> {
        if external_port == 0 {
            return Box::new(future::err(AddPortError::ExternalPortZeroInvalid));
        }
        if local_addr.port() == 0 {
            return Box::new(future::err(AddPortError::InternalPortZeroInvalid));
        }
        let future = self.add_port_mapping(
            protocol,
            external_port,
            local_addr,
            lease_duration,
            description,
        ).map_err(|err| match err {
                RequestError::ErrorCode(605, _) => AddPortError::DescriptionTooLong,
                RequestError::ErrorCode(606, _) => AddPortError::ActionNotAuthorized,
                RequestError::ErrorCode(718, _) => AddPortError::PortInUse,
                RequestError::ErrorCode(724, _) => AddPortError::SamePortValuesRequired,
                RequestError::ErrorCode(725, _) => AddPortError::OnlyPermanentLeasesSupported,
                e => AddPortError::RequestError(e),
            });
        Box::new(future)
    }

    /// Remove a port mapping.
    pub fn remove_port(
        &self,
        protocol: PortMappingProtocol,
        external_port: u16,
    ) -> Box<Future<Item = (), Error = RemovePortError>> {
        let header = "\"urn:schemas-upnp-org:service:WANIPConnection:1#DeletePortMapping\"";
        let body = format!(
            "<?xml version=\"1.0\"?>
        <s:Envelope xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/\" s:encodingStyle=\"http://schemas.xmlsoap.org/soap/encoding/\">
        <s:Body>
            <u:DeletePortMapping xmlns:u=\"urn:schemas-upnp-org:service:WANIPConnection:1\">
                <NewProtocol>{}</NewProtocol>
                <NewExternalPort>{}</NewExternalPort>
                <NewRemoteHost></NewRemoteHost>
            </u:DeletePortMapping>
        </s:Body>
        </s:Envelope>
        ",
            protocol,
            external_port
        );

        let future = self.perform_request(header, &*body, "DeletePortMappingResponse")
            .map(|_| ())
            .map_err(|err| match err {
                RequestError::ErrorCode(606, _) => RemovePortError::ActionNotAuthorized,
                RequestError::ErrorCode(714, _) => RemovePortError::NoSuchPortMapping,
                e => RemovePortError::RequestError(e),
            });
        Box::new(future)
    }
}

impl fmt::Display for Gateway {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "http://{}{}", self.addr, self.control_url)
    }
}

impl PartialEq for Gateway {
    fn eq(&self, other: &Gateway) -> bool {
        self.addr == other.addr && self.control_url == other.control_url
    }
}

impl Eq for Gateway {}

impl Hash for Gateway {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.addr.hash(state);
        self.control_url.hash(state);
    }
}

fn parse_response(text: String, ok: &str) -> Result<(String, xmltree::Element), RequestError> {
    let mut xml = match xmltree::Element::parse(text.as_bytes()) {
        Ok(xml) => xml,
        Err(..) => return Err(RequestError::InvalidResponse(text)),
    };
    let mut body = match xml.get_mut_child("Body") {
        Some(body) => body,
        None => return Err(RequestError::InvalidResponse(text)),
    };
    if let Some(ok) = body.take_child(ok) {
        return Ok((text, ok));
    }
    let upnp_error = match body.get_child("Fault")
        .and_then(|e| e.get_child("detail"))
        .and_then(|e| e.get_child("UPnPError")) {
        Some(upnp_error) => upnp_error,
        None => return Err(RequestError::InvalidResponse(text)),
    };
    match (
        upnp_error.get_child("errorCode"),
        upnp_error.get_child("errorDescription"),
    ) {
        (Some(e), Some(d)) => {
            match (e.text.as_ref(), d.text.as_ref()) {
                (Some(et), Some(dt)) => {
                    match et.parse::<u16>() {
                        Ok(en) => Err(RequestError::ErrorCode(en, From::from(&dt[..]))),
                        Err(..) => Err(RequestError::InvalidResponse(text)),
                    }
                }
                _ => Err(RequestError::InvalidResponse(text)),
            }
        }
        _ => Err(RequestError::InvalidResponse(text)),
    }
}
