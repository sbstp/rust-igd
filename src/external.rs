use std::error::{Error, FromError};
use std::fmt::{self, Display, Formatter};
use std::old_io::net::ip::IpAddr;
use std::str;

use curl::ErrCode;
use curl::http;

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

// RequestError::InvalidResponse text.
const ERR_INVALID_RESPONSE: &'static str = "Invalid response received from router";

// Errors
pub enum RequestError {
    ErrCode(ErrCode),
    InvalidResponse,
}

impl Display for RequestError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            RequestError::ErrCode(ref err) => err.fmt(f),
            RequestError::InvalidResponse => write!(f, "{}", ERR_INVALID_RESPONSE),
        }
    }
}

impl FromError<ErrCode> for RequestError {
    fn from_error(err: ErrCode) -> RequestError {
        RequestError::ErrCode(err)
    }
}

impl Error for RequestError {
    fn description(&self) -> &str {
        match *self {
            RequestError::ErrCode(ref err) => err.description(),
            RequestError::InvalidResponse => ERR_INVALID_RESPONSE,
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            RequestError::ErrCode(ref err) => err.cause(),
            RequestError::InvalidResponse => None,
        }
    }
}

// Get the external IP address.
pub fn get_external_ip(url: &str) -> Result<IpAddr, RequestError>  {
    let resp = try!(http::handle()
        .post(url, EXTERNAL_IP_REQUEST)
        .header("SOAPAction", SOAP_ACTION)
        .exec());
    let text = str::from_utf8(resp.get_body()).unwrap(); // TODO Shouldn't, but can fail.
    extract_address(text)
}

// Extract the address from the text.
fn extract_address(text: &str) -> Result<IpAddr, RequestError> {
    let re = regex!(r"<NewExternalIPAddress>(\d+\.\d+\.\d+\.\d+)</NewExternalIPAddress>");
    match re.captures(text) {
        None => Err(RequestError::InvalidResponse),
        Some(cap) => {
            match cap.at(1) {
                None => Err(RequestError::InvalidResponse),
                Some(ip) => Ok(ip.parse::<IpAddr>().unwrap()),
            }
        },
    }
}
