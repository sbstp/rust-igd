use crate::PortMappingProtocol;
use std::net::SocketAddrV4;

// Content of the request.
pub const SEARCH_REQUEST: &'static str = "M-SEARCH * HTTP/1.1\r
Host:239.255.255.250:1900\r
ST:urn:schemas-upnp-org:device:InternetGatewayDevice:1\r
Man:\"ssdp:discover\"\r
MX:3\r\n\r\n";

pub const GET_EXTERNAL_IP_HEADER: &'static str =
    r#""urn:schemas-upnp-org:service:WANIPConnection:1#GetExternalIPAddress""#;

pub const ADD_ANY_PORT_MAPPING_HEADER: &'static str =
    r#""urn:schemas-upnp-org:service:WANIPConnection:1#AddAnyPortMapping""#;

pub const ADD_PORT_MAPPING_HEADER: &'static str = r#""urn:schemas-upnp-org:service:WANIPConnection:1#AddPortMapping""#;

pub const DELETE_PORT_MAPPING_HEADER: &'static str =
    r#""urn:schemas-upnp-org:service:WANIPConnection:1#DeletePortMapping""#;

pub const GET_GENERIC_PORT_MAPPING_ENTRY: &'static str =
    r#""urn:schemas-upnp-org:service:WANIPConnection:1#GetGenericPortMappingEntry""#;

const MESSAGE_HEAD: &'static str = r#"<?xml version="1.0"?>
<s:Envelope s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/" xmlns:s="http://schemas.xmlsoap.org/soap/envelope/">
<s:Body>"#;

const MESSAGE_TAIL: &'static str = r#"</s:Body>
</s:Envelope>"#;

fn format_message(body: String) -> String {
    format!("{}{}{}", MESSAGE_HEAD, body, MESSAGE_TAIL)
}

pub fn format_get_external_ip_message() -> String {
    format!(
        r#"<?xml version="1.0"?>
<s:Envelope s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/" xmlns:s="http://schemas.xmlsoap.org/soap/envelope/">
    <s:Body>
        <m:GetExternalIPAddress xmlns:m="urn:schemas-upnp-org:service:WANIPConnection:1">
        </m:GetExternalIPAddress>
    </s:Body>
</s:Envelope>"#
    )
}

pub fn format_add_any_port_mapping_message(
    protocol: PortMappingProtocol,
    external_port: u16,
    local_addr: SocketAddrV4,
    lease_duration: u32,
    description: &str,
) -> String {
    format_message(format!(
        r#"<u:AddAnyPortMapping xmlns:u="urn:schemas-upnp-org:service:WANIPConnection:1">
        <NewProtocol>{}</NewProtocol>
        <NewExternalPort>{}</NewExternalPort>
        <NewInternalClient>{}</NewInternalClient>
        <NewInternalPort>{}</NewInternalPort>
        <NewLeaseDuration>{}</NewLeaseDuration>
        <NewPortMappingDescription>{}</NewPortMappingDescription>
        <NewEnabled>1</NewEnabled>
        <NewRemoteHost></NewRemoteHost>
        </u:AddAnyPortMapping>"#,
        protocol,
        external_port,
        local_addr.ip(),
        local_addr.port(),
        lease_duration,
        description,
    ))
}

pub fn format_add_port_mapping_message(
    protocol: PortMappingProtocol,
    external_port: u16,
    local_addr: SocketAddrV4,
    lease_duration: u32,
    description: &str,
) -> String {
    format_message(format!(
        r#"<u:AddPortMapping xmlns:u="urn:schemas-upnp-org:service:WANIPConnection:1">
        <NewProtocol>{}</NewProtocol>
        <NewExternalPort>{}</NewExternalPort>
        <NewInternalClient>{}</NewInternalClient>
        <NewInternalPort>{}</NewInternalPort>
        <NewLeaseDuration>{}</NewLeaseDuration>
        <NewPortMappingDescription>{}</NewPortMappingDescription>
        <NewEnabled>1</NewEnabled>
        <NewRemoteHost></NewRemoteHost>
        </u:AddPortMapping>"#,
        protocol,
        external_port,
        local_addr.ip(),
        local_addr.port(),
        lease_duration,
        description,
    ))
}

pub fn format_delete_port_message(protocol: PortMappingProtocol, external_port: u16) -> String {
    format_message(format!(
        r#"<u:DeletePortMapping xmlns:u="urn:schemas-upnp-org:service:WANIPConnection:1">
        <NewProtocol>{}</NewProtocol>
        <NewExternalPort>{}</NewExternalPort>
        <NewRemoteHost></NewRemoteHost>
        </u:DeletePortMapping>"#,
        protocol,
        external_port
    ))
}

pub fn formate_get_generic_port_mapping_entry_message(port_mapping_index: u32) -> String {
    format_message(format!(
        r#"<u:GetGenericPortMappingEntry xmlns:u="urn:schemas-upnp-org:service:WANIPConnection:1">
        <NewPortMappingIndex>{}</NewPortMappingIndex>
        </u:GetGenericPortMappingEntry>"#,
        port_mapping_index
    ))
}
