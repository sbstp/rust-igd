use std::error::{Error, FromError};
use std::fmt::{self, Display, Formatter};
use std::old_io::IoError;

use hyper::client::Client;
use hyper::header::Headers;
use hyper::HttpError;

// Content of the request.
const EXTERNAL_IP_REQUEST: &'static str =
"<SOAP-ENV:Envelope SOAP-ENV:encodingStyle=\"http://schemas.xmlsoap.org/soap/encoding/\" xmlns:SOAP-ENV=\"http://schemas.xmlsoap.org/soap/envelope/\">
    <SOAP-ENV:Body>
        <m:GetExternalIPAddress xmlns:m=\"urn:schemas-upnp-org:service:WANIPConnection:1\">
        </m:GetExternalIPAddress>
    </SOAP-ENV:Body>
</SOAP-ENV:Envelope>";

// Content of the SOAPAction header.
const SOAP_ACTION: &'static str = "\"urn:schemas-upnp-org:service:WANIPConnection:1#GetExternalIPAddress\"";

// Errors
pub enum RequestError {
    HttpError(HttpError),
    InvalidResponse,
    IoError(IoError),
}

impl Display for RequestError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            RequestError::HttpError(ref err) => err.fmt(f),
            RequestError::InvalidResponse => write!(f, "Invalid response received from router."), // TODO const
            RequestError::IoError(ref err) => err.fmt(f),
        }
    }
}

impl FromError<IoError> for RequestError {
    fn from_error(err: IoError) -> RequestError {
        RequestError::IoError(err)
    }
}

impl FromError<HttpError> for RequestError {
    fn from_error(err: HttpError) -> RequestError {
        RequestError::HttpError(err)
    }
}

impl Error for RequestError {
    fn description(&self) -> &str {
        match *self {
            RequestError::HttpError(ref err) => err.description(),
            RequestError::InvalidResponse => "Invalid response received from router.", // TODO const
            RequestError::IoError(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            RequestError::HttpError(ref err) => err.cause(),
            RequestError::InvalidResponse => None,
            RequestError::IoError(ref err) => err.cause(),
        }
    }
}

// Get the external IP address.
// TODO return IpAddr instead of String
pub fn get_external_ip(url: &str) -> Result<String, RequestError>  {
    let mut client = Client::new();

    let mut headers = Headers::new();
    headers.set_raw("SOAPAction", vec![String::from_str(SOAP_ACTION).into_bytes()]); // TODO clean-up

    let mut builder = client.post(url);
    builder = builder.headers(headers);
    builder = builder.body(EXTERNAL_IP_REQUEST);

    let mut res = try!(builder.send());
    let text = try!(res.read_to_string());
    extract_address(text)
}

// Extract the address from the text.
fn extract_address(text: String) -> Result<String, RequestError> {
    let re = regex!(r"<NewExternalIPAddress>(\d+\.\d+\.\d+\.\d+)</NewExternalIPAddress>");
    match re.captures(text.as_slice()) {
        None => Err(RequestError::InvalidResponse),
        Some(cap) => {
            match cap.at(1) {
                None => Err(RequestError::InvalidResponse),
                Some(ip) => Ok(ip.to_string()),
            }
        },
    }
}
