//! Implements <https://datatracker.ietf.org/doc/rfc1035/>

mod domain;
pub mod message;
pub mod punycode;
mod resource_type;

use crate::resource_type::{ResourceRecordClass, ResourceRecordType};
pub use domain::Domain;

use message::Consume;

use std::net::{IpAddr, Ipv4Addr, UdpSocket};

use crate::message::Message;

use anyhow::{Context, Result};
use thiserror::Error;

const MAX_DATAGRAM_SIZE: usize = 1024;
const UDP_SOCKET: &str = "0.0.0.0:20000";
const MAX_RESOLUTION_STEPS: usize = 5;

/// The root server used to resolve domains.
/// See [this list of root servers](https://www.iana.org/domains/root/servers).
const ROOT_SERVER: IpAddr = IpAddr::V4(Ipv4Addr::new(199, 7, 83, 42));

#[derive(Debug, Error)]
pub enum DNSError {
    #[error("Invalid DNS response")]
    InvalidResponse,
    #[error("Could not resolve {:?}", .0)]
    CouldNotResolve(Domain),
    #[error("Invalid domain: {:?}", .0)]
    InvalidDomain(Domain),
    #[error("Maximum dns resolution steps exceeded")]
    MaxResolutionStepsExceeded,
}

pub fn resolve(domain: &Domain) -> Result<IpAddr> {
    let mut nameserver = ROOT_SERVER;

    // incrementally resolve segments
    // www.ecosia.com will be resolved in the following order
    // 1) com
    // 2) ecosia.com
    // 3) www.ecosia.com
    for _ in 0..MAX_RESOLUTION_STEPS {
        let message = try_resolve_from(nameserver, domain)?;

        // Check if the response contains our answer
        if let Some(ip) = message.get_answer(domain) {
            return Ok(ip);
        }

        // Check if the response contains the domain name of an authoritative nameserver
        if let Some(ns_domain) = message.get_authority(domain) {
            // resolve that nameserver's domain and then
            // continue trying to resolve from that ns
            nameserver = resolve(&ns_domain)?;
            continue;
        }

        // We did not make any progress
        return Err(DNSError::CouldNotResolve(domain.clone()).into());
    }
    Err(DNSError::MaxResolutionStepsExceeded.into())
}

fn try_resolve_from(nameserver: IpAddr, domain: &Domain) -> Result<Message> {
    // Bind a UDP socket
    let socket = UdpSocket::bind(UDP_SOCKET)?;
    socket
        .connect((nameserver, 53))
        .with_context(|| format!("Connecting to nameserver {nameserver:?}"))?;

    // Send a DNS query
    let message = Message::new(domain);
    let expected_id = message.header.id;

    let mut bytes = vec![0; message.size()];
    message.write_to_buffer(&mut bytes);
    socket.send(&bytes)?;

    // Read the DNS response
    let mut response = [0; MAX_DATAGRAM_SIZE];
    let response_length = socket.recv(&mut response)?;

    let (parsed_message, _) =
        Message::read(&response[..response_length], 0).map_err(|_| DNSError::InvalidResponse)?;

    if parsed_message.header.id != expected_id {
        return Err(DNSError::InvalidResponse.into());
    }

    Ok(parsed_message)
}
