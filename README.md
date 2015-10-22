## Internet Gateway Device client

This is a simple library that communicates with an UPNP enabled gateway device (a router). Contributions and feedback are welcome.
At the moment, you can search for the gateway, request the gateway's external address and, add/remove port mappings. See the `examples/` folder for a demo.

Contributions are welcome! This is pretty delicate to test, please submit an issue if you have trouble using this.

## API

```rust
// Bind the UDP socket to all interfaces
fn search_gateway() -> Result<Gateway, SearchError>
// Bind the UDP socket to the given ip address
fn search_gateway_from(ip: Ipv4Addr) -> Result<Gateway, SearchError>
// Bind the UDP socket to all interfaces. Search with timeout.
fn search_gateway_timeout(timeout: Duration) -> Result<Gateway, SearchError>
// Bind the UDP socket to the given ip address. Search with timeout.
fn search_gateway_from_timeout(ip: Ipv4Addr, timeout: Duration) -> Result<Gateway, SearchError>

// Gateway struct
pub struct Gateway {
    pub addr: SocketAddrV4,
    pub control_url: String,
}
// Gateway methods
impl Gateway {
    // Get the gateway's external ip address
    pub fn get_external_ip(&self) -> Result<Ipv4Addr, RequestError>;
    // Add a port mapping
    pub fn add_port(&self, protocol: PortMappingProtocol,
                    external_port: u16, local_addr: SocketAddrV4, lease_duration: u32,
                    description: &str)
                    -> Result<(), RequestError>;
    // Remove a port mapping
    pub fn remove_port(&self, protocol: PortMappingProtocol,
                       external_port: u16)
                       -> Result<(), RequestError>;
}
```

## License
MIT
