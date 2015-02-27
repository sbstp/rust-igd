#![feature(plugin)]
#![feature(old_io)]
#![feature(io)]
#![feature(core)]
#![plugin(regex_macros)]

extern crate curl;
extern crate regex;

pub use self::external::get_external_ip;

// search of gateway
pub use self::search::search_gateway;
pub use self::search::SearchError;

mod external;
mod search;
