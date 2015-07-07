use std::net::{SocketAddr, SocketAddrV4, Ipv4Addr};

extern crate igd;

fn main() {
    match igd::search_gateway() {
        Err(ref err) => match *err {
            igd::SearchError::IoError(ref ioe) => println!("IoError: {}", ioe),
            _ => println!("{:?}", err),
        },
        Ok(gateway) => {
            let local_addr = "192.168.1.2".parse::<Ipv4Addr>().unwrap();
            let local_addr = SocketAddr::V4(SocketAddrV4::new(local_addr,
                                                              8080u16));

            match igd::add_port(&gateway, igd::PortMappingProtocol::TCP, 80,
                                local_addr, 0, "crust") {
                Err(ref err) => match *err {
                    igd::RequestError::IoError(ref ioe) => {
                        println!("IoError: {}", ioe)
                    },
                    _ => println!("{:?}", err),
                },
                Ok(()) => {
                    println!("It worked");
                },
            }
        },
    }
}
