use std::io;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::fmt;
use std;
use rand::distributions::IndependentSample;

use errors::{RequestError,GetExternalIpError,AddPortError,AddAnyPortError,RemovePortError};
use ::{PortMappingProtocol};
use async::{Gateway as AsyncGateway};


/// This structure represents a gateway found by the search functions.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Gateway {
    /// Socket address of the gateway
    pub addr: SocketAddrV4,
    /// Control url of the device
    pub control_url: String,

    inner: AsyncGateway
}

impl Gateway {

    /// Get the external IP address of the gateway.
    pub fn get_external_ip(&self) -> Result<Ipv4Addr, GetExternalIpError> {
        self.inner.get_external_ip().wait()
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
    pub fn get_any_address(&self,
                           protocol: PortMappingProtocol,
                           local_addr: SocketAddrV4,
                           lease_duration: u32,
                           description: &str)
            -> Result<SocketAddrV4, AddAnyPortError>
    {
        self.inner.get_any_address(protocol, local_addr, lease_duration, description).wait()
    }


    /// Add a port mapping.with any external port.
    ///
    /// The local_addr is the address where the traffic is sent to.
    /// The lease_duration parameter is in seconds. A value of 0 is infinite.
    ///
    /// # Returns
    ///
    /// The external port that was mapped on success. Otherwise an error.
    pub fn add_any_port(&self, protocol: PortMappingProtocol,
                        local_addr: SocketAddrV4,
                        lease_duration: u32, description: &str)
            -> Result<u16, AddAnyPortError>
    {
        self.inner.add_any_port(protocol, local_addr, lease_duration).wait()
    }

    /// Add a port mapping.
    ///
    /// The local_addr is the address where the traffic is sent to.
    /// The lease_duration parameter is in seconds. A value of 0 is infinite.
    pub fn add_port(&self, protocol: PortMappingProtocol,
                    external_port: u16, local_addr: SocketAddrV4, lease_duration: u32,
                    description: &str) -> Result<(), AddPortError> {
        self.inner.add_port(protocol, external_port, local_addr, lease_duration, description).wait()
    }

    /// Remove a port mapping.
    pub fn remove_port(&self, protocol: PortMappingProtocol,
                       external_port: u16) -> Result<(), RemovePortError> {
        self.inner.remove_port(protocol, external_port).wait()
    }
}

impl fmt::Display for Gateway {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "http://{}{}", self.addr, self.control_url)
    }
}
