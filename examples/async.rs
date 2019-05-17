//! IGD async API example.
//!
//! It demonstrates how to:
//! * get external IP
//! * add port mappings
//! * remove port mappings
//!
//! If everything works fine, 2 port mappings are added, 1 removed and we're left with single
//! port mapping: External 1234 ---> 4321 Internal

extern crate igd;
extern crate futures;
extern crate tokio;
extern crate simplelog;

use std::env;
use std::net::SocketAddrV4;

use igd::async::{search_gateway, SearchOptions};
use igd::PortMappingProtocol;
use futures::future::Future;
use simplelog::{SimpleLogger, LevelFilter, Config as LogConfig};

fn main() {
    let ip = match env::args().nth(1) {
        Some(ip) => ip,
        None => {
            println!("Local socket address is missing!");
            println!("This example requires a socket address representing the local machine and the port to bind to as an argument");
            println!("Example: target/debug/examples/async 192.168.0.198:4321");
            println!("Example: cargo run --features async --example async -- 192.168.0.198:4321");
            return;
        }
    };
    let ip: SocketAddrV4 = ip.parse().expect("Invalid socket address");

    let _ = SimpleLogger::init(LevelFilter::Debug, LogConfig::default());

    let f = futures::lazy(move || {
        search_gateway(SearchOptions::default())
        .map_err(|e| panic!("Failed to find IGD: {}", e))
        .and_then(move |gateway| gateway.get_external_ip()
            .map_err(|e| panic!("Failed to get external IP: {}", e))
            .and_then(|ip| Ok((gateway, ip)))
        )
        .and_then(|(gateway, pub_ip)| {
            println!("Our public IP: {}", pub_ip);
            Ok(gateway)
        })
        .and_then(move |gateway| {
            gateway.add_port(
                PortMappingProtocol::TCP,
                1234,
                ip,
                120,
                "rust-igd-async-example",
            )
            .map_err(|e| panic!("Failed to add port mapping: {}", e))
            .and_then(|_| {
                println!("New port mapping was successfully added.");
                Ok(gateway)
            })
        })
        .and_then(move |gateway| {
            gateway.add_port(
                PortMappingProtocol::TCP,
                2345,
                ip,
                120,
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
        })

    }).map(|_| () ).map_err(|_| () );

    tokio::run(f);
}
