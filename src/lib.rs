#![feature(plugin)]
#![feature(old_io)]
#![feature(io)]
#![feature(core)]
#![plugin(regex_macros)]

extern crate curl;
extern crate regex;

// request external ip address
pub use self::external::get_external_ip;
pub use self::external::RequestError;

// search of gateway
pub use self::search::search_gateway;
pub use self::search::SearchError;

mod external;
mod search;
