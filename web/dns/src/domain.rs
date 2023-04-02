use crate::{message::Consume, punycode::idna_encode, DNSError};

use std::fmt;

const DOMAIN_MAX_SEGMENTS: u8 = 10;

#[derive(PartialEq, Clone)]
pub struct Domain(Vec<String>);

impl fmt::Debug for Domain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.join("."))
    }
}

impl Domain {
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
    pub fn decode(source: &[u8]) -> Result<Self, DNSError> {
        let mut domain = Self(vec![]);

        let mut ptr = 0;
        loop {
            let block_size = source[ptr] as usize;

            if block_size == 0x00 {
                break;
            }

            ptr += 1;

            if source.len() <= ptr + block_size {
                // Domain block reaches out of bounds
                return Err(DNSError::InvalidResponse);
            }

            domain.add_segment(String::from_utf8_lossy(&source[ptr..ptr + block_size]).to_string());

            ptr += block_size;
        }

        Ok(domain)
    }
}

impl Consume for Domain {
    fn read(global_buffer: &[u8], ptr: usize) -> Result<(Self, usize), ()> {
        let buffer = &global_buffer[ptr..];

        let mut result: Vec<String> = vec![];

        let mut ptr = 0;
        let mut num_segments = 0;

        loop {
            if num_segments > DOMAIN_MAX_SEGMENTS {
                return Err(());
            }

            // Check if it is a pointer to part of another domain
            // See https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.4
            if buffer[ptr] >> 6 == 0b11 {
                // Compression
                let compress_ptr =
                    u16::from_be_bytes(buffer[ptr..ptr + 2].try_into().unwrap()) & !(0b11 << 14);
                let (referenced_domain, _) = Self::read(global_buffer, compress_ptr as usize)?;

                result.extend(referenced_domain.0);

                ptr += 2;
                break; // No continuation after compress pointer
            } else {
                // No Compression
                let block_length = buffer[ptr] as usize;
                ptr += 1;

                if block_length == 0x00 {
                    break;
                }

                assert!(
                    ptr + block_length < buffer.len(),
                    "domain block reaches out of bounds"
                );

                let block_data = std::str::from_utf8(&buffer[ptr..ptr + block_length])
                    .map_err(|_| ())?
                    .to_string();
                result.push(block_data);

                ptr += block_length;
            }

            num_segments += 1;
        }
        Ok((Domain(result), ptr))
    }
}
