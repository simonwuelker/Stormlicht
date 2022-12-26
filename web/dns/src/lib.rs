//! Implements <https://datatracker.ietf.org/doc/rfc1035/>

pub mod message;
pub mod punycode;
mod resource_type;

use crate::message::Domain;
use crate::resource_type::{ResourceRecordClass, ResourceRecordType};

use message::Consume;

use std::net::{IpAddr, UdpSocket};

use crate::message::Message;

const MAX_DATAGRAM_SIZE: usize = 1024;
const UDP_SOCKET: &'static str = "0.0.0.0:20000";
const NAMESERVER: &'static str = "8.8.8.8:53";

#[derive(Debug)]
pub enum DNSError {
    FailedToBindSocket,
    ConnectionRefused,
    InvalidResponse,
    NetworkError,
}

pub fn resolve(domain_name: &[u8]) -> Result<IpAddr, DNSError> {
    // Bind a UDP socket
    let socket = UdpSocket::bind(UDP_SOCKET).map_err(|_| DNSError::FailedToBindSocket)?;
    socket.connect(NAMESERVER).unwrap(); // .map_err(|_| DNSError::ConnectionRefused)?;

    // Send a DNS query
    let message = Message::new(domain_name);
    let expected_id = message.header.id;

    let mut bytes = vec![0; message.size()];
    message.write_to_buffer(&mut bytes);
    socket.send(&bytes).map_err(|_| DNSError::NetworkError)?;

    // Read the DNS response
    let mut response = [0; MAX_DATAGRAM_SIZE];
    let response_length = socket
        .recv(&mut response)
        .map_err(|_| DNSError::NetworkError)?;
    let (parsed_message, _) =
        Message::read(&response[..response_length], 0).map_err(|_| DNSError::InvalidResponse)?;

    if parsed_message.header.id != expected_id {
        return Err(DNSError::InvalidResponse);
    }

    parsed_message
        .get(&Domain::new(domain_name))
        .map_err(|_| DNSError::InvalidResponse)
}
