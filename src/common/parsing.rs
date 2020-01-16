use std::io;
use std::net::{Ipv4Addr, SocketAddrV4};

use url::Url;
use xmltree::Element;

use crate::PortMappingProtocol;
use crate::errors::{AddAnyPortError, AddPortError, GetExternalIpError, RemovePortError, GetGenericPortMappingEntryError, RequestError, SearchError};

// Parse the result.
pub fn parse_search_result(text: &str) -> Result<(SocketAddrV4, String), SearchError> {
    use SearchError::InvalidResponse;

    for line in text.lines() {
        let line = line.trim();
        if line.to_ascii_lowercase().starts_with("location:") {
            if let Some(colon) = line.find(":") {
                let url_text = &line[colon + 1..].trim();
                let url = Url::parse(url_text).map_err(|_| InvalidResponse)?;
                let addr: Ipv4Addr = url
                    .host_str()
                    .ok_or(InvalidResponse)
                    .and_then(|s| s.parse().map_err(|_| InvalidResponse))?;
                let port: u16 = url.port_or_known_default().ok_or(InvalidResponse)?;

                return Ok((SocketAddrV4::new(addr, port), url.path().to_string()));
            }
        }
    }
    Err(InvalidResponse)
}

pub fn parse_control_url<R>(resp: R) -> Result<String, SearchError>
where
    R: io::Read,
{
    let root = Element::parse(resp)?;

    let device = root.get_child("device").ok_or(SearchError::InvalidResponse)?;
    if let Ok(control_url) = parse_control_url_scan_device(&device) {
        return Ok(control_url);
    }

    return Err(SearchError::InvalidResponse);
}

fn parse_control_url_scan_device(device: &Element) -> Result<String, SearchError> {
    let service_list = device.get_child("serviceList").ok_or(SearchError::InvalidResponse)?;
    for service in &service_list.children {
        if service.name == "service" {
            if let Some(service_type) = service.get_child("serviceType") {
                if service_type.text.as_ref().map(|s| s.as_str())
                    == Some("urn:schemas-upnp-org:service:WANPPPConnection:1") || service_type.text.as_ref().map(|s| s.as_str())
                    == Some("urn:schemas-upnp-org:service:WANIPConnection:1")
                {
                    if let Some(control_url) = service.get_child("controlURL") {
                        if let Some(text) = &control_url.text {
                            return Ok(text.clone());
                        }
                    }
                }
            }
        }
    }

    let device_list = device.get_child("deviceList").ok_or(SearchError::InvalidResponse)?;
    for sub_device in &device_list.children {
        if sub_device.name == "device" {
            if let Ok(control_url) = parse_control_url_scan_device(&sub_device) {
                return Ok(control_url);
            }
        }
    }

    return Err(SearchError::InvalidResponse);
}

pub struct RequestReponse {
    text: String,
    xml: xmltree::Element,
}

pub type RequestResult = Result<RequestReponse, RequestError>;

pub fn parse_response(text: String, ok: &str) -> RequestResult {
    let mut xml = match xmltree::Element::parse(text.as_bytes()) {
        Ok(xml) => xml,
        Err(..) => return Err(RequestError::InvalidResponse(text)),
    };
    let body = match xml.get_mut_child("Body") {
        Some(body) => body,
        None => return Err(RequestError::InvalidResponse(text)),
    };
    if let Some(ok) = body.take_child(ok) {
        return Ok(RequestReponse { text: text, xml: ok });
    }
    let upnp_error = match body
        .get_child("Fault")
        .and_then(|e| e.get_child("detail"))
        .and_then(|e| e.get_child("UPnPError"))
    {
        Some(upnp_error) => upnp_error,
        None => return Err(RequestError::InvalidResponse(text)),
    };

    match (
        upnp_error.get_child("errorCode"),
        upnp_error.get_child("errorDescription"),
    ) {
        (Some(e), Some(d)) => match (e.text.as_ref(), d.text.as_ref()) {
            (Some(et), Some(dt)) => match et.parse::<u16>() {
                Ok(en) => Err(RequestError::ErrorCode(en, From::from(&dt[..]))),
                Err(..) => Err(RequestError::InvalidResponse(text)),
            },
            _ => Err(RequestError::InvalidResponse(text)),
        },
        _ => Err(RequestError::InvalidResponse(text)),
    }
}

pub fn parse_get_external_ip_response(result: RequestResult) -> Result<Ipv4Addr, GetExternalIpError> {
    match result {
        Ok(resp) => match resp
            .xml
            .get_child("NewExternalIPAddress")
            .and_then(|e| e.text.as_ref())
            .and_then(|t| t.parse::<Ipv4Addr>().ok())
        {
            Some(ipv4_addr) => Ok(ipv4_addr),
            None => Err(GetExternalIpError::RequestError(RequestError::InvalidResponse(
                resp.text,
            ))),
        },
        Err(RequestError::ErrorCode(606, _)) => Err(GetExternalIpError::ActionNotAuthorized),
        Err(e) => Err(GetExternalIpError::RequestError(e)),
    }
}

pub fn parse_add_any_port_mapping_response(result: RequestResult) -> Result<u16, Option<AddAnyPortError>> {
    match result {
        Ok(resp) => {
            match resp
                .xml
                .get_child("NewReservedPort")
                .and_then(|e| e.text.as_ref())
                .and_then(|t| t.parse::<u16>().ok())
            {
                Some(port) => Ok(port),
                None => Err(Some(AddAnyPortError::RequestError(RequestError::InvalidResponse(
                    resp.text,
                )))),
            }
        }
        Err(err) => Err(match err {
            RequestError::ErrorCode(401, _) => None,
            RequestError::ErrorCode(605, _) => Some(AddAnyPortError::DescriptionTooLong),
            RequestError::ErrorCode(606, _) => Some(AddAnyPortError::ActionNotAuthorized),
            RequestError::ErrorCode(728, _) => Some(AddAnyPortError::NoPortsAvailable),
            e => Some(AddAnyPortError::RequestError(e)),
        }),
    }
}

pub fn convert_add_random_port_mapping_error(error: RequestError) -> Option<AddAnyPortError> {
    match error {
        RequestError::ErrorCode(724, _) => None,
        RequestError::ErrorCode(605, _) => Some(AddAnyPortError::DescriptionTooLong),
        RequestError::ErrorCode(606, _) => Some(AddAnyPortError::ActionNotAuthorized),
        RequestError::ErrorCode(718, _) => Some(AddAnyPortError::NoPortsAvailable),
        RequestError::ErrorCode(725, _) => Some(AddAnyPortError::OnlyPermanentLeasesSupported),
        e => Some(AddAnyPortError::RequestError(e)),
    }
}

pub fn convert_add_same_port_mapping_error(error: RequestError) -> AddAnyPortError {
    match error {
        RequestError::ErrorCode(606, _) => AddAnyPortError::ActionNotAuthorized,
        RequestError::ErrorCode(718, _) => AddAnyPortError::ExternalPortInUse,
        RequestError::ErrorCode(725, _) => AddAnyPortError::OnlyPermanentLeasesSupported,
        e => AddAnyPortError::RequestError(e),
    }
}

pub fn convert_add_port_error(err: RequestError) -> AddPortError {
    match err {
        RequestError::ErrorCode(605, _) => AddPortError::DescriptionTooLong,
        RequestError::ErrorCode(606, _) => AddPortError::ActionNotAuthorized,
        RequestError::ErrorCode(718, _) => AddPortError::PortInUse,
        RequestError::ErrorCode(724, _) => AddPortError::SamePortValuesRequired,
        RequestError::ErrorCode(725, _) => AddPortError::OnlyPermanentLeasesSupported,
        e => AddPortError::RequestError(e),
    }
}

pub fn parse_delete_port_mapping_response(result: RequestResult) -> Result<(), RemovePortError> {
    match result {
        Ok(_) => Ok(()),
        Err(err) => Err(match err {
            RequestError::ErrorCode(606, _) => RemovePortError::ActionNotAuthorized,
            RequestError::ErrorCode(714, _) => RemovePortError::NoSuchPortMapping,
            e => RemovePortError::RequestError(e),
        }),
    }
}

/// One port mapping entry as returned by GetGenericPortMappingEntry
pub struct PortMappingEntry {
    /// The remote host for which the mapping is valid
    /// Can be an IP address or a host name
    pub remote_host: String,
    /// The external port of the mapping
    pub external_port: u16,
    /// The protocol of the mapping
    pub protocol: PortMappingProtocol,
    /// The internal (local) port
    pub internal_port: u16,
    /// The internal client of the port mapping
    /// Can be an IP address or a host name
    pub internal_client: String,
    /// A flag whether this port mapping is enabled
    pub enabled: bool,
    /// A description for this port mapping
    pub port_mapping_description: String,
    /// The lease duration of this port mapping in seconds
    pub lease_duration: u32,
}

pub fn parse_get_generic_port_mapping_entry(result: RequestResult) -> Result<PortMappingEntry, GetGenericPortMappingEntryError> {
    let response = result?;
    let xml = response.xml;
    let make_err = |msg: String| || GetGenericPortMappingEntryError::RequestError(RequestError::InvalidResponse(msg));
    let extract_field = |field: &str| xml.get_child(field).ok_or_else(make_err(format!("{} is missing", field)));
    let remote_host = extract_field("NewRemoteHost")?.text.clone().unwrap_or("".into());
    let external_port =  extract_field("NewExternalPort")?.text.as_ref().and_then(|t| t.parse::<u16>().ok()).ok_or_else(make_err("Field NewExternalPort is invalid".into()))?;
    let protocol = match extract_field("NewProtocol")?.text.as_ref().map(String::as_ref) {
        Some("UDP") => PortMappingProtocol::UDP,
        Some("TCP") => PortMappingProtocol::TCP,
        _ => return Err(GetGenericPortMappingEntryError::RequestError(RequestError::InvalidResponse("Field NewProtocol is invalid".into()))),
    };
    let internal_port = extract_field("NewInternalPort")?.text.as_ref().and_then(|t| t.parse::<u16>().ok()).ok_or_else(make_err("Field NewInternalPort is invalid".into()))?;
    let internal_client = extract_field("NewInternalClient")?.text.clone().ok_or_else(make_err("Field NewInternalClient is empty".into()))?;
    let enabled = match extract_field("NewEnabled")?.text.as_ref().and_then(|t| t.parse::<u16>().ok()).ok_or_else(make_err("Field Enabled is invalid".into()))? {
        0 => false,
        1 => true,
        _ => return Err(GetGenericPortMappingEntryError::RequestError(RequestError::InvalidResponse("Field NewEnabled is invalid".into())))
    };
    let port_mapping_description = extract_field("NewPortMappingDescription")?.text.clone().unwrap_or("".into());
    let lease_duration = extract_field("NewLeaseDuration")?.text.as_ref().and_then(|t| t.parse::<u32>().ok()).ok_or_else(make_err("Field NewLeaseDuration is invalid".into()))?;
    Ok(PortMappingEntry{ remote_host, external_port, protocol, internal_port, internal_client, enabled, port_mapping_description, lease_duration })
}

#[test]
fn test_parse_search_result_case_insensitivity() {
    assert!(parse_search_result("location:http://0.0.0.0:0/control_url").is_ok());
    assert!(parse_search_result("LOCATION:http://0.0.0.0:0/control_url").is_ok());
}

#[test]
fn test_parse_search_result_ok() {
    let result = parse_search_result("location:http://0.0.0.0:0/control_url").unwrap();
    assert_eq!(result.0.ip(), &Ipv4Addr::new(0, 0, 0, 0));
    assert_eq!(result.0.port(), 0);
    assert_eq!(&result.1[..], "/control_url");
}

#[test]
fn test_parse_search_result_fail() {
    assert!(parse_search_result("content-type:http://0.0.0.0:0/control_url").is_err());
}

#[test]
fn test_parse_device1() {
    let text = r#"<?xml version="1.0" encoding="UTF-8"?>
<root xmlns="urn:schemas-upnp-org:device-1-0">
   <specVersion>
      <major>1</major>
      <minor>0</minor>
   </specVersion>
   <device>
      <deviceType>urn:schemas-upnp-org:device:InternetGatewayDevice:1</deviceType>
      <friendlyName></friendlyName>
      <manufacturer></manufacturer>
      <manufacturerURL></manufacturerURL>
      <modelDescription></modelDescription>
      <modelName></modelName>
      <modelNumber>1</modelNumber>
      <serialNumber>00000000</serialNumber>
      <UDN></UDN>
      <serviceList>
         <service>
            <serviceType>urn:schemas-upnp-org:service:Layer3Forwarding:1</serviceType>
            <serviceId>urn:upnp-org:serviceId:Layer3Forwarding1</serviceId>
            <controlURL>/ctl/L3F</controlURL>
            <eventSubURL>/evt/L3F</eventSubURL>
            <SCPDURL>/L3F.xml</SCPDURL>
         </service>
      </serviceList>
      <deviceList>
         <device>
            <deviceType>urn:schemas-upnp-org:device:WANDevice:1</deviceType>
            <friendlyName>WANDevice</friendlyName>
            <manufacturer>MiniUPnP</manufacturer>
            <manufacturerURL>http://miniupnp.free.fr/</manufacturerURL>
            <modelDescription>WAN Device</modelDescription>
            <modelName>WAN Device</modelName>
            <modelNumber>20180615</modelNumber>
            <modelURL>http://miniupnp.free.fr/</modelURL>
            <serialNumber>00000000</serialNumber>
            <UDN>uuid:804e2e56-7bfe-4733-bae0-04bf6d569692</UDN>
            <UPC>MINIUPNPD</UPC>
            <serviceList>
               <service>
                  <serviceType>urn:schemas-upnp-org:service:WANCommonInterfaceConfig:1</serviceType>
                  <serviceId>urn:upnp-org:serviceId:WANCommonIFC1</serviceId>
                  <controlURL>/ctl/CmnIfCfg</controlURL>
                  <eventSubURL>/evt/CmnIfCfg</eventSubURL>
                  <SCPDURL>/WANCfg.xml</SCPDURL>
               </service>
            </serviceList>
            <deviceList>
               <device>
                  <deviceType>urn:schemas-upnp-org:device:WANConnectionDevice:1</deviceType>
                  <friendlyName>WANConnectionDevice</friendlyName>
                  <manufacturer>MiniUPnP</manufacturer>
                  <manufacturerURL>http://miniupnp.free.fr/</manufacturerURL>
                  <modelDescription>MiniUPnP daemon</modelDescription>
                  <modelName>MiniUPnPd</modelName>
                  <modelNumber>20180615</modelNumber>
                  <modelURL>http://miniupnp.free.fr/</modelURL>
                  <serialNumber>00000000</serialNumber>
                  <UDN>uuid:804e2e56-7bfe-4733-bae0-04bf6d569692</UDN>
                  <UPC>MINIUPNPD</UPC>
                  <serviceList>
                     <service>
                        <serviceType>urn:schemas-upnp-org:service:WANIPConnection:1</serviceType>
                        <serviceId>urn:upnp-org:serviceId:WANIPConn1</serviceId>
                        <controlURL>/ctl/IPConn</controlURL>
                        <eventSubURL>/evt/IPConn</eventSubURL>
                        <SCPDURL>/WANIPCn.xml</SCPDURL>
                     </service>
                  </serviceList>
               </device>
            </deviceList>
         </device>
      </deviceList>
      <presentationURL>http://192.168.0.1/</presentationURL>
   </device>
</root>"#;

    assert_eq!(parse_control_url(text.as_bytes()).unwrap(), "/ctl/IPConn");
}
