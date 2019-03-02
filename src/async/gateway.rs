use std::fmt;
use std::hash::{Hash, Hasher};
use std::net::{Ipv4Addr, SocketAddrV4};

use super::soap;
use errors::{AddAnyPortError, AddPortError, GetExternalIpError, RemovePortError, RequestError};
use futures::future;
use futures::Future;
use tokio_core::reactor::Handle;
use tokio_retry::strategy::FixedInterval;
use tokio_retry::{Error as RetryError, RetryIf};

use common;
use common::parsing::RequestReponse;
use common::{messages, parsing};
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
    ) -> Box<Future<Item = RequestReponse, Error = RequestError>> {
        let url = format!("{}", self);
        let ok = ok.to_owned();
        let future = soap::send_async(&url, soap::Action::new(header), body, &self.handle)
            .map_err(|err| RequestError::from(err))
            .and_then(move |text| parsing::parse_response(text, &ok));
        Box::new(future)
    }

    /// Get the external IP address of the gateway in a tokio compatible way
    pub fn get_external_ip(&self) -> Box<Future<Item = Ipv4Addr, Error = GetExternalIpError>> {
        let future = self
            .perform_request(
                messages::GET_EXTERNAL_IP_HEADER,
                &messages::format_get_external_ip_message(),
                "GetExternalIPAddressResponse",
            )
            .then(|result| parsing::parse_get_external_ip_response(result));
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
        let future = self
            .get_external_ip()
            .map_err(|err| AddAnyPortError::from(err))
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

        let external_port = common::random_port();

        let gateway = self.clone();
        let description = description.to_owned();

        // First, attempt to call the AddAnyPortMapping method.
        let future = self
            .perform_request(
                messages::ADD_ANY_PORT_MAPPING_HEADER,
                &messages::format_add_any_port_mapping_message(
                    protocol,
                    external_port,
                    local_addr,
                    lease_duration,
                    &description,
                ),
                "AddAnyPortMappingResponse",
            )
            .then(
                move |result| match parsing::parse_add_any_port_mapping_response(result) {
                    Ok(port) => Box::new(future::ok(port)),
                    Err(None) => {
                        // The router does not have the AddAnyPortMapping method.
                        // Fall back to using AddPortMapping with a random port.
                        gateway.retry_add_random_port_mapping(protocol, local_addr, lease_duration, &description)
                    }
                    Err(Some(err)) => Box::new(future::err(err)),
                },
            );
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
            move || gateway.add_random_port_mapping(protocol, local_addr, lease_duration, &description),
            |err: &AddAnyPortError| match err {
                &AddAnyPortError::NoPortsAvailable => true,
                _ => false,
            },
        )
        .map_err(|err| match err {
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

        let external_port = common::random_port();

        let future = self
            .add_port_mapping(protocol, external_port, local_addr, lease_duration, &description)
            .map(move |_| external_port)
            .or_else(move |err| match parsing::convert_add_random_port_mapping_error(err) {
                Some(err) => Box::new(future::err(err)),
                // The router requires that internal and external ports be the same.
                None => gateway.add_same_port_mapping(protocol, local_addr, lease_duration, &description),
            });

        Box::new(future)
    }

    fn add_same_port_mapping(
        &self,
        protocol: PortMappingProtocol,
        local_addr: SocketAddrV4,
        lease_duration: u32,
        description: &str,
    ) -> Box<Future<Item = u16, Error = AddAnyPortError>> {
        let future = self
            .add_port_mapping(protocol, local_addr.port(), local_addr, lease_duration, description)
            .map(move |_| local_addr.port())
            .map_err(|err| parsing::convert_add_same_port_mapping_error(err));

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
        let future = self
            .perform_request(
                messages::ADD_PORT_MAPPING_HEADER,
                &messages::format_add_port_mapping_message(
                    protocol,
                    external_port,
                    local_addr,
                    lease_duration,
                    description,
                ),
                "AddPortMappingResponse",
            )
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

        let future = self
            .add_port_mapping(protocol, external_port, local_addr, lease_duration, description)
            .map_err(|err| parsing::convert_add_port_error(err));

        Box::new(future)
    }

    /// Remove a port mapping.
    pub fn remove_port(
        &self,
        protocol: PortMappingProtocol,
        external_port: u16,
    ) -> Box<Future<Item = (), Error = RemovePortError>> {
        let future = self
            .perform_request(
                messages::DELETE_PORT_MAPPING_HEADER,
                &messages::format_delete_port_message(protocol, external_port),
                "DeletePortMappingResponse",
            )
            .then(|result| parsing::parse_delete_port_mapping_response(result));
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
