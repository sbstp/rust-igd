use std::fmt;

use futures::future;
use futures::{Future, Stream};
use hyper;
use hyper::header::{ContentLength, ContentType, Formatter, Header, Raw};
use hyper::{Client, Post, Request};
use tokio_core::reactor::Handle;

use errors::RequestError;

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

pub fn send_async(
    url: &str,
    action: Action,
    body: &str,
    handle: &Handle,
) -> Box<Future<Item = String, Error = RequestError>> {
    let client = Client::new(&handle);
    let uri = match url.parse() {
        Ok(uri) => uri,
        Err(err) => return Box::new(future::err(RequestError::from(err))),
    };
    let mut req = Request::new(Post, uri);
    req.headers_mut().set(action);
    req.headers_mut().set(ContentType::xml());
    req.headers_mut().set(ContentLength(body.len() as u64));
    req.set_body(body.to_owned());
    let future = client
        .request(req)
        .and_then(|resp| resp.body().concat2())
        .map_err(|err| RequestError::from(err))
        .and_then(|bytes| String::from_utf8(bytes.to_vec()).map_err(|err| RequestError::from(err)));
    Box::new(future)
}
