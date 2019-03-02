use std::net::SocketAddrV4;
use PortMappingProtocol;

pub const GET_EXTERNAL_IP_HEADER: &'static str =
    r#""urn:schemas-upnp-org:service:WANIPConnection:1#GetExternalIPAddress""#;

pub const ADD_ANY_PORT_MAPPING_HEADER: &'static str =
    r#""urn:schemas-upnp-org:service:WANIPConnection:1#AddAnyPortMapping""#;

pub const ADD_PORT_MAPPING_HEADER: &'static str = r#""urn:schemas-upnp-org:service:WANIPConnection:1#AddPortMapping""#;

pub const DELETE_PORT_MAPPING_HEADER: &'static str =
    r#""urn:schemas-upnp-org:service:WANIPConnection:1#DeletePortMapping""#;

pub fn format_get_external_ip_message() -> String {
    format!(r#"<?xml version="1.0"?>
<SOAP-ENV:Envelope SOAP-ENV:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/" xmlns:SOAP-ENV="http://schemas.xmlsoap.org/soap/envelope/">
    <SOAP-ENV:Body>
        <m:GetExternalIPAddress xmlns:m="urn:schemas-upnp-org:service:WANIPConnection:1">
        </m:GetExternalIPAddress>
    </SOAP-ENV:Body>
</SOAP-ENV:Envelope>"#)
}

pub fn format_add_any_port_mapping_message(
    protocol: PortMappingProtocol,
    external_port: u16,
    local_addr: SocketAddrV4,
    lease_duration: u32,
    description: &str,
) -> String {
    format!("<?xml version=\"1.0\"?>
<s:Envelope xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/\" s:encodingStyle=\"http://schemas.xmlsoap.org/soap/encoding/\">
<s:Body>
    <u:AddAnyPortMapping xmlns:u=\"urn:schemas-upnp-org:service:WANIPConnection:1\">
        <NewProtocol>{}</NewProtocol>
        <NewExternalPort>{}</NewExternalPort>
        <NewInternalClient>{}</NewInternalClient>
        <NewInternalPort>{}</NewInternalPort>
        <NewLeaseDuration>{}</NewLeaseDuration>
        <NewPortMappingDescription>{}</NewPortMappingDescription>
        <NewEnabled>1</NewEnabled>
        <NewRemoteHost></NewRemoteHost>
    </u:AddAnyPortMapping>
</s:Body>
</s:Envelope>",
        protocol,
        external_port,
        local_addr.ip(),
        local_addr.port(),
        lease_duration,
        description,
    )
}

pub fn format_add_port_mapping_message(
    protocol: PortMappingProtocol,
    external_port: u16,
    local_addr: SocketAddrV4,
    lease_duration: u32,
    description: &str,
) -> String {
    format!("<?xml version=\"1.0\"?>
<s:Envelope xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/\" s:encodingStyle=\"http://schemas.xmlsoap.org/soap/encoding/\">
<s:Body>
    <u:AddPortMapping xmlns:u=\"urn:schemas-upnp-org:service:WANIPConnection:1\">
        <NewProtocol>{}</NewProtocol>
        <NewExternalPort>{}</NewExternalPort>
        <NewInternalClient>{}</NewInternalClient>
        <NewInternalPort>{}</NewInternalPort>
        <NewLeaseDuration>{}</NewLeaseDuration>
        <NewPortMappingDescription>{}</NewPortMappingDescription>
        <NewEnabled>1</NewEnabled>
        <NewRemoteHost></NewRemoteHost>
    </u:AddPortMapping>
</s:Body>
</s:Envelope>",
        protocol,
        external_port,
        local_addr.ip(),
        local_addr.port(),
        lease_duration,
        description,
    )
}

pub fn format_delete_port_message(protocol: PortMappingProtocol, external_port: u16) -> String {
    format!("<?xml version=\"1.0\"?>
<s:Envelope xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/\" s:encodingStyle=\"http://schemas.xmlsoap.org/soap/encoding/\">
<s:Body>
    <u:DeletePortMapping xmlns:u=\"urn:schemas-upnp-org:service:WANIPConnection:1\">
        <NewProtocol>{}</NewProtocol>
        <NewExternalPort>{}</NewExternalPort>
        <NewRemoteHost></NewRemoteHost>
    </u:DeletePortMapping>
</s:Body>
</s:Envelope>",
        protocol,
        external_port
    )
}
