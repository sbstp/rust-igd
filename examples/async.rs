//! IGD async API example.
//!
//! It demonstrates how to:
//! * get external IP
//! * add port mappings
//! * remove port mappings
//!
//! If everything works fine, 2 port mappings are added, 1 removed and we're left with single
//! port mapping: External 1234 ---> 4321 Internal

extern crate futures;
extern crate igd;
extern crate tokio_core;

use futures::future::Future;
use igd::tokio::search_gateway;
use igd::PortMappingProtocol;

fn main() {
    let mut evloop = tokio_core::reactor::Core::new().unwrap();
    let handle = evloop.handle();

    let task = search_gateway(&handle)
        .map_err(|e| panic!("Failed to find IGD: {}", e))
        .and_then(|gateway| {
            gateway
                .get_external_ip()
                .map_err(|e| panic!("Failed to get external IP: {}", e))
                .and_then(|ip| Ok((gateway, ip)))
        })
        .and_then(|(gateway, pub_ip)| {
            println!("Our public IP: {}", pub_ip);
            Ok(gateway)
        })
        .and_then(|gateway| {
            gateway
                .add_port(
                    PortMappingProtocol::TCP,
                    1234,
                    "192.168.1.210:4321".parse().unwrap(),
                    0,
                    "rust-igd-async-example",
                )
                .map_err(|e| panic!("Failed to add port mapping: {}", e))
                .and_then(|_| {
                    println!("New port mapping was successfully added.");
                    Ok(gateway)
                })
        })
        .and_then(|gateway| {
            gateway
                .add_port(
                    PortMappingProtocol::TCP,
                    2345,
                    "192.168.1.210:5432".parse().unwrap(),
                    0,
                    "rust-igd-async-example",
                )
                .map_err(|e| panic!("Failed to add port mapping: {}", e))
                .and_then(|_| {
                    println!("New port mapping was successfully added.");
                    Ok(gateway)
                })
        })
        .and_then(|gateway| gateway.remove_port(PortMappingProtocol::TCP, 2345))
        .and_then(|_| {
            println!("Port was removed.");
            Ok(())
        });

    let _ = evloop.run(task).unwrap();
}
