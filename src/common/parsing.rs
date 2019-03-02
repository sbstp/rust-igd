use std::io;

use xmltree::Element;

use errors::{RequestError, SearchError};

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
