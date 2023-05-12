//! Implements <https://datatracker.ietf.org/doc/rfc1035/>

#![feature(lazy_cell)]

mod dns_cache;
mod domain;
pub mod message;
pub mod punycode;
mod resource_type;

use crate::resource_type::{ResourceRecordClass, ResourceRecordType};
pub use dns_cache::DNS_CACHE;
pub use domain::Domain;

use message::Consume;

use std::{
    io,
    net::{IpAddr, Ipv4Addr, UdpSocket},
};

use crate::message::Message;

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

/// Resolve a domain name.
///
/// If the domain name is inside the DNS cache, no actual resolution
/// is performed.
pub fn lookup(domain: &Domain) -> Result<IpAddr, DNSError> {
    DNS_CACHE.get(domain)
}

/// Resolve a domain name by contacting the DNS server.
///
/// Returns a tuple of `(resolved IP, TTL in seconds)`.
///
/// This function **does not** make use of a cache.
/// You should prefer [lookup] instead.
fn resolve(domain: &Domain) -> Result<(IpAddr, u32), DNSError> {
    let mut nameserver = ROOT_SERVER;

    // incrementally resolve segments
    // www.ecosia.com will be resolved in the following order
    // 1) com
    // 2) ecosia.com
    // 3) www.ecosia.com
    for _ in 0..MAX_RESOLUTION_STEPS {
        let message = try_resolve_from(nameserver, domain)?;

        // Check if the response contains our answer
        if let Some((ip, ttl)) = message.get_answer(domain) {
            return Ok((ip, ttl));
        }

        // Check if the response contains the domain name of an authoritative nameserver
        if let Some(ns_domain) = message.get_authority(domain) {
            // resolve that nameserver's domain and then
            // continue trying to resolve from that ns
            nameserver = DNS_CACHE.get(&ns_domain)?;
            continue;
        }

        // We did not make any progress
        return Err(DNSError::CouldNotResolve(domain.clone()));
    }
    Err(DNSError::MaxResolutionStepsExceeded)
}

fn try_resolve_from(nameserver: IpAddr, domain: &Domain) -> Result<Message, DNSError> {
    // Bind a UDP socket
    let socket = UdpSocket::bind(UDP_SOCKET).map_err(DNSError::IO)?;
    socket.connect((nameserver, 53)).map_err(DNSError::IO)?;

    // Send a DNS query
    let message = Message::new(domain);
    let expected_id = message.header.id;

    let mut bytes = vec![0; message.size()];
    message.write_to_buffer(&mut bytes);
    socket.send(&bytes).map_err(DNSError::IO)?;

    // Read the DNS response
    let mut response = [0; MAX_DATAGRAM_SIZE];
    let response_length = socket.recv(&mut response).map_err(DNSError::IO)?;

    let (parsed_message, _) =
        Message::read(&response[..response_length], 0).map_err(|_| DNSError::InvalidResponse)?;

    if parsed_message.header.id != expected_id {
        return Err(DNSError::UnexpectedID);
    }

    Ok(parsed_message)
}
