## Internet Gateway Device client

This is a simple library that communicates with an UPNP enabled gateway device (a router). Contributions and feedback are welcome.
Currently, you can only search the gateway and request the gateway's address. See the `examples/` folder for a demo.

Contributions are welcome! This is pretty delicate to test, please submit an issue if you have trouble using this.

## API

```rust
fn get_external_ip<A: ToSocketAddr>(addr: A) -> Result<IpAddr, RequestError>
fn search_gateway() -> Result<SocketAddr, SearchError>
```

## License
MIT
