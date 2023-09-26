//! Implements <https://datatracker.ietf.org/doc/rfc1035/>

#![feature(lazy_cell)]

mod dns_cache;
mod domain;
pub mod message;
mod resource_type;

use crate::resource_type::{ResourceRecordClass, ResourceRecordType};
pub use dns_cache::DNS_CACHE;
pub use domain::Domain;

use std::{
    io,
    net::{IpAddr, Ipv4Addr},
};

const MAX_DATAGRAM_SIZE: usize = 1024;
const UDP_SOCKET: &str = "0.0.0.0:20000";
const MAX_RESOLUTION_STEPS: usize = 5;

/// The root server used to resolve domains.
/// See [this list of root servers](https://www.iana.org/domains/root/servers).
const ROOT_SERVER: IpAddr = IpAddr::V4(Ipv4Addr::new(199, 7, 83, 42));

#[derive(Debug)]
pub enum DNSError {
    InvalidResponse,
    CouldNotResolve(Domain),
    InvalidDomain(Domain),
    MaxResolutionStepsExceeded,
    UnexpectedID,
    IO(io::Error),
}

impl From<io::Error> for DNSError {
    fn from(value: io::Error) -> Self {
        Self::IO(value)
    }
}
