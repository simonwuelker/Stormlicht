use crate::{
    message::Message,
    reader::Reader,
    resource_type::{ResourceRecord, ResourceRecordClass},
    DNSError, DNS_CACHE, MAX_DATAGRAM_SIZE, MAX_RESOLUTION_STEPS, ROOT_SERVER, UDP_SOCKET,
};
use sl_std::{punycode::idna_encode, read::ReadExt};

use std::{
    fmt,
    io::Read,
    net::{IpAddr, UdpSocket},
};

const DOMAIN_MAX_SEGMENTS: u8 = 10;
const DNS_PORT: u16 = 53;

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Domain(Vec<String>);

impl fmt::Debug for Domain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.join("."))
    }
}

impl Domain {
    #[must_use]
    pub fn new(source: &str) -> Self {
        let mut segments = vec![];
        for segment in source.split('.') {
            segments.push(idna_encode(segment).unwrap());
        }

        Self(segments)
    }

    pub fn add_segment(&mut self, segment: String) {
        self.0.push(segment);
    }

    /// Encodes a domain name for use in a DNS query.
    ///
    /// Blocks are seperated by a byte specifying their length.
    /// The last byte is guaranteed to be a null byte
    ///
    /// # Example
    /// ```
    /// # use dns::Domain;
    /// let domain = Domain::new("www.example.com");
    /// let encoded_name = domain.encode();
    ///
    /// assert_eq!(&encoded_name, b"\x03www\x07example\x03com\x00");
    /// ```
    pub fn encode(&self) -> Vec<u8> {
        let length = self.0.len() + self.0.iter().map(|segment| segment.len()).sum::<usize>() + 1;
        let mut result = vec![0; length];

        let mut ptr = 0;
        for segment in &self.0 {
            debug_assert!(segment.is_ascii());

            result[ptr] = segment.len() as u8;
            ptr += 1;
            result[ptr..ptr + segment.len()].copy_from_slice(segment.as_bytes());
            ptr += segment.len();
        }
        debug_assert_eq!(ptr, length - 1);

        result
    }

    /// Decodes a domain name from a DNS query.
    ///
    /// Blocks are seperated by a byte specifying their length.
    /// The last byte is guaranteed to be a null byte
    ///
    /// # Example
    /// ```
    /// # use dns::Domain;
    /// let encoded_name = b"\x03www\x07example\x03com\x00";
    /// let domain_name = Domain::decode(encoded_name).unwrap();
    ///
    /// assert_eq!(domain_name, Domain::new("www.example.com"));
    /// ```
    ///
    /// # Panics
    /// This function panics if the given byte buffer is not a valid encoded domain name,
    /// for example `\x03www\x07example\x04com`.
    pub fn read_from(reader: &mut Reader) -> Result<Self, DNSError> {
        let mut result: Vec<String> = vec![];

        let mut num_segments = 0;
        loop {
            if num_segments > DOMAIN_MAX_SEGMENTS {
                return Err(DNSError::DomainTooLong);
            }

            let leading_byte = reader.read_be_u8()?;

            // Check if it is a pointer to part of another domain
            // See https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.4
            if leading_byte >> 6 == 0b11 {
                // Compression
                let compress_ptr =
                    ((leading_byte as u16 & 0b00111111) << 8) | reader.read_be_u8()? as u16;

                let referenced_domain = reader.domain_at(compress_ptr as u64)?;

                result.extend(referenced_domain.0);

                break; // No continuation after compress pointer
            } else {
                // No Compression
                let block_length = leading_byte as usize;

                if block_length == 0x00 {
                    break;
                }

                let mut buffer = vec![0; block_length];
                reader.read_exact(&mut buffer)?;
                let block_data =
                    String::from_utf8(buffer).map_err(|_| DNSError::InvalidResponse)?;

                result.push(block_data);
            }

            num_segments += 1;
        }

        Ok(Domain(result))
    }

    /// Resolve a domain name.
    ///
    /// If the domain name is inside the DNS cache, no actual resolution
    /// is performed.
    #[inline]
    pub fn lookup(&self) -> Result<IpAddr, DNSError> {
        DNS_CACHE.get(self)
    }

    /// Resolve a domain name by contacting the DNS server.
    ///
    /// Returns a tuple of `(resolved IP, TTL in seconds)`.
    ///
    /// This function **does not** make use of a cache.
    /// You should prefer [lookup] instead.
    pub(crate) fn resolve(&self) -> Result<(IpAddr, u32), DNSError> {
        let mut nameserver = ROOT_SERVER;

        // incrementally resolve segments
        // www.ecosia.com will be resolved in the following order
        // 1) com
        // 2) ecosia.com
        // 3) www.ecosia.com
        for _ in 0..MAX_RESOLUTION_STEPS {
            let message = self.try_resolve_from(nameserver)?;

            // Check if the response contains our answer
            if let Some((ip, ttl)) = message.get_answer(self) {
                return Ok((ip, ttl));
            }

            // Insert any additional records provided by the server into our cache
            message
                .additional_records()
                .iter()
                .filter(|resource| resource.class == ResourceRecordClass::IN)
                .for_each(|resource| {
                    let referenced_ip = match resource.record {
                        ResourceRecord::A { ipv4 } => Some(IpAddr::V4(ipv4)),
                        ResourceRecord::AAAA { ipv6 } => Some(IpAddr::V6(ipv6)),
                        _ => None,
                    };

                    if let Some(ip) = referenced_ip {
                        DNS_CACHE.insert(resource.domain.clone(), ip, resource.time_to_live);
                    }
                });

            // Check if the response contains the domain name of an authoritative nameserver
            if let Some(ns_domain) = message.get_authority(self) {
                // resolve that nameserver's domain and then
                // continue trying to resolve from that ns
                nameserver = DNS_CACHE.get(&ns_domain)?;
            } else {
                // We did not make any progress
                return Err(DNSError::CouldNotResolve(self.clone()));
            }
        }
        Err(DNSError::MaxResolutionStepsExceeded)
    }

    fn try_resolve_from(&self, nameserver: IpAddr) -> Result<Message, DNSError> {
        // Bind a UDP socket
        let socket = UdpSocket::bind(UDP_SOCKET)?;
        socket.connect((nameserver, DNS_PORT))?;

        // Send a DNS query
        let message = Message::new(self);
        let expected_id = message.id();

        let mut bytes = vec![0; message.size()];
        message.write_to_buffer(&mut bytes);
        socket.send(&bytes)?;

        // Read the DNS response
        let mut response = [0; MAX_DATAGRAM_SIZE];
        let response_length = socket.recv(&mut response)?;

        let mut reader = Reader::new(&response[..response_length]);
        let parsed_message = Message::read_from(&mut reader)?;

        if parsed_message.id() != expected_id {
            return Err(DNSError::UnexpectedID);
        }

        Ok(parsed_message)
    }
}
