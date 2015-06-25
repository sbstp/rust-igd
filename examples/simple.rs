extern crate igd;

fn main() {
    match igd::search_gateway() {
        Err(ref err) => println!("{:?}", err),
        Ok(local_soaddr) => {
            match igd::get_external_ip(local_soaddr) {
                Err(ref err) => println!("{:?}", err),
                Ok(ext_addr) => {
                    println!("Local gateway: {}, External ip address: {}", local_soaddr, ext_addr);
                },
            }
        },
    }
}
