extern crate igd;

fn main() {
    match igd::search_gateway() {
        Err(ref err) => println!("{:?}", err),
        Ok(gateway) => {
            match gateway.get_external_ip() {
                Err(ref err) => println!("{:?}", err),
                Ok(ext_addr) => {
                    println!("Local gateway: {}, External ip address: {}", gateway, ext_addr);
                },
            }
        },
    }
}
