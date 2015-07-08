use std::fmt;
use std::io::{self, Read};

use hyper;
use hyper::client::Client;
use hyper::error::Error as HyperError;
use hyper::header::{Header, HeaderFormat};

#[derive(Clone, Debug)]
pub struct Action(String);

impl Action {

    pub fn new(action: &str) -> Action {
        Action(action.into())
    }

}

impl Header for Action {

    fn header_name() -> &'static str {
        "SOAPAction"
    }

    fn parse_header(raw: &[Vec<u8>]) -> hyper::Result<Action> {
        // Can SOAPAction appear more than once?
        if raw.len() == 1 {
            match String::from_utf8(raw[0].clone()) {
                Ok(action) => return Ok(Action(action)),
                Err(_) => return Err(HyperError::Header),
            }
        }
        Err(HyperError::Header)
    }

}

impl HeaderFormat for Action {

    fn fmt_header(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }

}

pub enum Error {
    HttpError(HyperError),
    IoError(io::Error),
}

impl From<HyperError> for Error {
    fn from(err: HyperError) -> Error {
        Error::HttpError(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IoError(err)
    }
}

pub fn send(url: &str, action: Action, body: &str) -> Result<String, Error>  {
    let client = Client::new();
    let mut resp = try!(client.post(url)
        .header(action)
        .body(body)
        .send());

    let mut text = String::new();
    try!(resp.read_to_string(&mut text));
    Ok(text)
}
