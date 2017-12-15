use std::net::{Ipv4Addr, SocketAddrV4};
use std::fmt;
use tokio_core::reactor::Core;
use futures::Future;

use errors::{GetExternalIpError, AddPortError, AddAnyPortError, RemovePortError};
use PortMappingProtocol;
use async::Gateway as AsyncGateway;


/// This structure represents a gateway found by the search functions.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Gateway {
    /// Socket address of the gateway
    pub addr: SocketAddrV4,
    /// Control url of the device
    pub control_url: String,
}

impl Gateway {
    /// Get the external IP address of the gateway.
    pub fn get_external_ip(&self) -> Result<Ipv4Addr, GetExternalIpError> {
        let mut core = Core::new().unwrap();
        let async = AsyncGateway::new(self.addr, self.control_url.clone(), core.handle());
        core.run(async.get_external_ip::<Box<
            Future<
                Item = Ipv4Addr,
                Error = GetExternalIpError,
            >,
        >>())
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
    ) -> Result<SocketAddrV4, AddAnyPortError> {
        let mut core = Core::new().unwrap();
        let async = AsyncGateway::new(self.addr, self.control_url.clone(), core.handle());
        core.run(async.get_any_address(
            protocol,
            local_addr,
            lease_duration,
            description,
        ))
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
    ) -> Result<u16, AddAnyPortError> {
        let mut core = Core::new().unwrap();
        let async = AsyncGateway::new(self.addr, self.control_url.clone(), core.handle());
        core.run(async.add_any_port(
            protocol,
            local_addr,
            lease_duration,
            description,
        ))
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
    ) -> Result<(), AddPortError> {
        let mut core = Core::new().unwrap();
        let async = AsyncGateway::new(self.addr, self.control_url.clone(), core.handle());
        core.run(async.add_port(
            protocol,
            external_port,
            local_addr,
            lease_duration,
            description,
        ))
    }

    /// Remove a port mapping.
    pub fn remove_port(
        &self,
        protocol: PortMappingProtocol,
        external_port: u16,
    ) -> Result<(), RemovePortError> {
        let mut core = Core::new().unwrap();
        let async = AsyncGateway::new(self.addr, self.control_url.clone(), core.handle());
        core.run(async.remove_port(protocol, external_port))
    }
}

impl fmt::Display for Gateway {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "http://{}{}", self.addr, self.control_url)
    }
}
