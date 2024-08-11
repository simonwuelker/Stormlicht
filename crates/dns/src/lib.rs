//! Implements <https://datatracker.ietf.org/doc/rfc1035/>

mod dns_cache;
mod domain;
pub mod message;
mod reader;
mod resource_type;

use crate::resource_type::{ResourceRecord, ResourceRecordClass};
pub use dns_cache::DNS_CACHE;
pub use domain::Domain;
use error_derive::Error;

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

#[derive(Debug, Error)]
pub enum DNSError {
    #[msg = "invalid response"]
    InvalidResponse,

    #[msg = "could not resolve"]
    CouldNotResolve,

    #[msg = "maximum number of resolution steps exceeded"]
    MaxResolutionStepsExceeded,

    #[msg = "unexpected id"]
    UnexpectedID,

    #[msg = "io error"]
    IO(io::Error),

    #[msg = "domain too long"]
    DomainTooLong,
}
