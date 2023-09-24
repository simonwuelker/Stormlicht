use std::net::{self};

use dns::DNSError;

#[derive(Clone, Debug)]
pub enum ServerName {
    Domain(String),
    IP(net::IpAddr),
}

impl From<String> for ServerName {
    fn from(value: String) -> Self {
        Self::Domain(value)
    }
}

impl From<net::IpAddr> for ServerName {
    fn from(value: net::IpAddr) -> Self {
        Self::IP(value)
    }
}

impl TryFrom<&ServerName> for net::IpAddr {
    type Error = DNSError;

    fn try_from(value: &ServerName) -> Result<net::IpAddr, Self::Error> {
        match value {
            ServerName::Domain(domain) => dns::Domain::new(domain.as_str()).lookup(),
            ServerName::IP(ip) => Ok(*ip),
        }
    }
}
