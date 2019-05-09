
use futures::{Future, Stream};

use hyper::header::{CONTENT_LENGTH, CONTENT_TYPE};
use hyper::{Request, Body, client::Client};

use errors::RequestError;

#[derive(Clone, Debug)]
pub struct Action(String);

impl Action {
    pub fn new(action: &str) -> Action {
        Action(action.into())
    }
}

const HEADER_NAME: &str = "SOAPAction";

pub fn send_async(
    url: &str,
    action: Action,
    body: &str,
) -> impl Future<Item = String, Error = RequestError> {
    use futures::future::{err, Either::A, Either::B};

    let client = Client::new();

    let req = Request::builder()
        .uri(url)
        .method("POST")
        .header(HEADER_NAME, action.0)
        .header(CONTENT_TYPE, "xml")
        .header(CONTENT_LENGTH, body.len() as u64)
        .body(Body::from(body.to_string()))
        .map_err(|e| RequestError::from(e) );

    let req = match req {
        Ok(r) => r,
        Err(e) => return A(err(e)),
    };

    let future = client
        .request(req)
        .and_then(|resp| resp.into_body().concat2())
        .map_err(|err| RequestError::from(err))
        .and_then(|bytes| String::from_utf8(bytes.to_vec())
        .map_err(|err| RequestError::from(err)));

    B(future)
}
