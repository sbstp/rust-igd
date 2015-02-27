#![feature(plugin)]
#![feature(old_io)]
#![feature(core)]
#![plugin(regex_macros)]

extern crate curl;
extern crate regex;

pub use self::external::get_external_ip;

mod external;
