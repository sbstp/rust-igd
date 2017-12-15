use std::fmt;
use std::string::FromUtf8Error;
use std::io;

use futures::{Future, Stream};
use futures::future;
use tokio_core::reactor::Handle;
use hyper;
use hyper::{Client, Request, Post};
use hyper::error::Error as HyperError;
use hyper::header::{Header, ContentType, Raw, Formatter};

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

    #[allow(unused_variables)]
    fn parse_header(raw: &Raw) -> hyper::Result<Action> {
        // Leave unimplemented as we shouldn't need it.
        unimplemented!();
    }

    fn fmt_header(&self, f: &mut Formatter) -> fmt::Result {
        f.fmt_line(&self.0)
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

impl From<hyper::error::UriError> for Error {
    fn from(err: hyper::error::UriError) -> Error {
        Error::HttpError(HyperError::from(err))
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IoError(err)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(err: FromUtf8Error) -> Error {
        Error::HttpError(HyperError::from(err))
    }
}

pub fn send_async(
    url: &str,
    action: Action,
    body: &str,
    handle: &Handle,
) -> Box<Future<Item = String, Error = Error>> {
    let client = Client::new(&handle);
    let uri = match url.parse() {
        Ok(uri) => uri,
        Err(err) => return Box::new(future::err(Error::from(err))),
    };
    let mut req = Request::new(Post, uri);
    req.headers_mut().set(action);
    req.headers_mut().set(ContentType::xml());
    req.set_body(body.to_owned());
    let future = client
        .request(req)
        .and_then(|resp| resp.body().concat2())
        .map_err(|err| Error::from(err))
        .and_then(|bytes| {
            String::from_utf8(bytes.to_vec()).map_err(|err| Error::from(err))
        });
    Box::new(future)
}
