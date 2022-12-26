//! https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1
use random::RNG;

use std::{vec, fmt, net::IpAddr};
use crate::{ResourceRecordType, ResourceRecordClass};

const DOMAIN_MAX_SEGMENTS: u8 = 10;

#[derive(Debug)]
pub enum QueryType {
    Standard = 0,
    Inverse = 1,
    Status = 2,
    Reserved,
}

#[derive(Debug)]
// https://datatracker.ietf.org/doc/html/rfc1035#section-4.1
pub(crate) struct Message {
    pub(crate) header: Header,
    pub(crate) question: Vec<Question>,
    pub(crate) answer: Vec<Resource>,
    pub(crate) _authority: Vec<Resource>,
    pub(crate) _additional: Vec<Resource>,
}

// https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1
pub struct Header {
    pub(crate) id: u16,
    flags: u16,
    num_questions: u16,
    num_answers: u16,
    num_authorities: u16,
    num_additional: u16,
}

#[derive(Debug)]
// https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.2
pub struct Question {
    domain: Domain,
    _query_type: QueryType,
    _query_class: (),
}

#[derive(Debug)]
// https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.3
pub struct Resource {
    domain: Domain,
    resource_type: ResourceRecordType,
    _class: ResourceRecordClass,
    _time_to_live: u32,
}

#[derive(Debug)]
pub enum ResponseCode {
    ///  No error condition
    Ok,

    /// Format error - The name server was unable to interpret the query.
    FormatError,

    /// Server failure - The name server was unable to process this query 
    /// due to a problem with the name server.
    ServerFailure,

    /// Name Error - Meaningful only for responses from an authoritative name
    /// server, this code signifies that the domain name referenced 
    /// in the query does not exist.
    NameError,

    /// Not Implemented - The name server does not support the requested kind of query.
    NotImplemented,

    /// Refused - The name server refuses to perform the specified operation for
    /// policy reasons.  For example, a name server may not wish to provide the
    /// information to the particular requester, or a name server may not
    /// wish to perform a particular operation (e.g., zone transfer) for particular data.
    Refused,

    /// Reserved for future use.
    Reserved,
}

impl Header {
    pub fn new(num_questions: u16) -> Self {
        Self {
            id: RNG::default().next_u16(),
            flags: 0x100,
            num_questions: num_questions,
            num_answers: 0x0000,
            num_authorities: 0x0000,
            num_additional: 0x000,
        }
    }

    pub fn write_to_buffer(&self, bytes: &mut [u8]){
        bytes[0..2].copy_from_slice(&self.id.to_be_bytes());
        bytes[2..4].copy_from_slice(&self.flags.to_be_bytes());
        bytes[4..6].copy_from_slice(&self.num_questions.to_be_bytes());
        bytes[6..8].copy_from_slice(&self.num_answers.to_be_bytes());
        bytes[8..10].copy_from_slice(&self.num_authorities.to_be_bytes());
        bytes[10..12].copy_from_slice(&self.num_additional.to_be_bytes());
    }

    pub fn recursion_available(&self) -> bool {
        (self.flags >> 8) & 1 != 0
    }

    pub fn recursion_desired(&self) -> bool {
        (self.flags >> 9) & 1 != 0
    }

    pub fn truncated(&self) -> bool {
        (self.flags >> 10) & 1 != 0
    }

    pub fn authoritative(&self) -> bool {
        (self.flags >> 11) & 1 != 0
    }

    pub fn response_code(&self) -> ResponseCode {
        match self.flags & 0b1111 {
            0 => ResponseCode::Ok,
            1 => ResponseCode::FormatError,
            2 => ResponseCode::ServerFailure,
            3 => ResponseCode::NameError,
            4 => ResponseCode::NotImplemented,
            5 => ResponseCode::Refused,
            _ => ResponseCode::Reserved,
        }
    }
}

impl Question {
    pub fn new(domain_bytes: Vec<u8>) -> Self {
        Self {
            domain: Domain(domain_bytes),
            _query_type: QueryType::Standard,
            _query_class: (),
        }
    }

    fn size(&self) -> usize {
        self.domain.encode().len() + 4
    }

    pub fn write_to_buffer(&self, bytes: &mut [u8]) -> usize {
        let encoded_domain = self.domain.encode();
        bytes[..encoded_domain.len()].copy_from_slice(&encoded_domain);

        let mut ptr = encoded_domain.len();

        bytes[ptr..ptr + 2].copy_from_slice(&1_u16.to_be_bytes());
        ptr += 2;

        bytes[ptr..ptr + 2].copy_from_slice(&1_u16.to_be_bytes());
        ptr += 2;
        ptr
    }
}

impl Message {
    pub fn new(domain_name: &[u8]) -> Self {
        Self {
            header: Header::new(1),
            question: vec![Question::new(domain_name.to_vec())],
            answer: vec![],
            _authority: vec![],
            _additional: vec![],
        }
    }

    pub fn size(&self) -> usize {
        16 + self.question.iter().map(|q| q.size()).sum::<usize>()
    }

    /// Serialize `self` into the provided byte buffer,
    /// returning the number of bytes that were written
    pub fn write_to_buffer(self, bytes: &mut [u8]) -> usize {
        self.header.write_to_buffer(&mut bytes[..12]);
        
        let mut ptr = 12;
        for question in &self.question {
            ptr += question.write_to_buffer(&mut bytes[ptr..]);
        }

        ptr
    }

    pub fn get(&self, domain: &Domain) -> Result<IpAddr, ()> {
        for answer in &self.answer {
            if answer.domain == *domain {
                match answer.resource_type {
                    ResourceRecordType::A{ ipv4 } => return Ok(IpAddr::V4(ipv4)),
                    ResourceRecordType::CNAME { ref alias } => {
                        return self.get(alias);
                    },
                    _ => {},
                }
            } 
        }

        // Our question was not answered
        Err(())
    }
}

#[derive(PartialEq)]
/// Stores the ascii bytes for an unencoded domain name
pub struct Domain(Vec<u8>);

impl fmt::Debug for Domain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &std::str::from_utf8(&self.0).unwrap())
    }
}

/// Generic trait for DNS-related things that can be read from a
/// byte buffer
pub(crate) trait Consume {
    /// Tries to consume the given data structure from a buffer at a given offset.
    /// Fails if the buffer is too small or the data is invalid.
    /// On success, returns the parsed data structure and the number of bytes (from ptr)
    /// that were read
    fn read(buffer: &[u8], ptr: usize) -> Result<(Self, usize), ()> where Self: Sized;
}

impl Consume for Message {
    fn read(buffer: &[u8], mut ptr: usize) -> Result<(Self, usize), ()> {
        let (header, bytes_read) = Header::read(&buffer, ptr)?;
        ptr += bytes_read;

        let mut questions = Vec::with_capacity(header.num_questions as usize);
        let mut answers = Vec::with_capacity(header.num_answers as usize);
        let mut authority = Vec::with_capacity(header.num_authorities as usize);
        let mut additional = Vec::with_capacity(header.num_additional as usize);

        println!("{header:?}");

        for _ in 0..header.num_questions {
            let (new_question, bytes_read) = Question::read(&buffer, ptr)?;
            questions.push(new_question);
            ptr += bytes_read;
        }

        for _ in 0..header.num_answers {
            let (new_answer, bytes_read) = Resource::read(&buffer, ptr)?;
            answers.push(new_answer);
            ptr += bytes_read;
        }

        for _ in 0..header.num_authorities {
            let (new_authority, bytes_read) = Resource::read(&buffer, ptr)?;
            authority.push(new_authority);
            ptr += bytes_read;
        }

        for _ in 0..header.num_additional {
            let (new_additional, bytes_read) = Resource::read(&buffer, ptr)?;
            additional.push(new_additional);
            ptr += bytes_read;
        }

        Ok((Self {
            header: header,
            question: questions,
            answer: answers,
            _authority: authority,
            _additional: additional,
        }, ptr))
    }
}


impl Consume for Question {
    fn read(global_buffer: &[u8], ptr: usize) -> Result<(Self, usize), ()> {
        let buffer = &global_buffer[ptr..];

        let (domain, domain_length) = Domain::read(global_buffer, ptr)?;
        
        let _query_type = u16::from_be_bytes(buffer[domain_length..domain_length + 2].try_into().unwrap());
        let _query_class = u16::from_be_bytes(buffer[domain_length + 2..domain_length + 4].try_into().unwrap());

        // FIXME properly parse the type/class
        Ok((Self {
            domain: domain,
            _query_type: QueryType::Standard,
            _query_class: (),
        }, domain_length +  4))
    }
}

impl Consume for Header {
    fn read(global_buffer: &[u8], ptr: usize) -> Result<(Self, usize), ()> {
        // ptr is, in practice, always zero but we still properly use it
        let buffer = &global_buffer[ptr..];

        if buffer.len() < 12 {
            return Err(());
        }
        let id = u16::from_be_bytes(buffer[0..2].try_into().unwrap());
        let flags = u16::from_be_bytes(buffer[2..4].try_into().unwrap());
        let num_questions = u16::from_be_bytes(buffer[4..6].try_into().unwrap());
        let num_answers = u16::from_be_bytes(buffer[6..8].try_into().unwrap());
        let num_authorities = u16::from_be_bytes(buffer[8..10].try_into().unwrap());
        let num_additional = u16::from_be_bytes(buffer[10..12].try_into().unwrap());

        Ok((Self {
            id,
            flags,
            num_questions,
            num_answers,
            num_authorities,
            num_additional
        }, 12))
    }
}

impl Consume for Domain {
    fn read(global_buffer: &[u8], ptr: usize) -> Result<(Self, usize), ()> {
        let buffer = &global_buffer[ptr..];

        let mut result: Vec<u8> = vec![];
        
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
                let compress_ptr = u16::from_be_bytes(buffer[ptr..ptr + 2].try_into().unwrap()) & !(0b11 << 14);
                let (referenced_domain, _) = Self::read(global_buffer, compress_ptr as usize)?;

                if num_segments != 0 {
                    result.push(0x2e);
                }
                
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

                assert!(ptr + block_length < buffer.len(), "domain block reaches out of bounds");

                let mut block_data = vec![0; block_length];
                block_data[..].copy_from_slice(&buffer[ptr..ptr + block_length]);
                
                if num_segments != 0 {
                    result.push(0x2e);
                }

                result.extend(block_data);

                ptr += block_length;
            }

            num_segments += 1;
        }
        Ok((Domain(result), ptr))
    }
}

impl Consume for Resource {
    fn read(global_buffer: &[u8], ptr: usize) -> Result<(Self, usize), ()> {
        let buffer = &global_buffer[ptr..];
        let (domain, domain_length) = Domain::read(global_buffer, ptr)?;

        let rtype = (global_buffer, ptr + domain_length).try_into()?;

        let class = u16::from_be_bytes(buffer[domain_length + 2..domain_length + 4].try_into().unwrap()).into();
        let ttl = u32::from_be_bytes(buffer[domain_length + 4..domain_length + 8].try_into().unwrap());
        let rdlength = u16::from_be_bytes(buffer[domain_length + 8..domain_length + 10].try_into().unwrap()) as usize;

        Ok((Self {
            domain: domain,
            resource_type: rtype,
            _class: class,
            _time_to_live: ttl,
        }, domain_length + 10 + rdlength))
    }
}

impl Domain {
    pub fn new(source: &[u8]) -> Self {
        Self (source.to_vec())
    }

    /// Encodes a domain name for use in a DNS query.
    /// 
    /// Blocks are seperated by a byte specifying their length.
    /// The last byte is guaranteed to be a null byte
    /// 
    /// # Example
    /// ```
    /// # use dns::message::Domain; 
    /// let domain = Domain::new(b"www.example.com");
    /// let encoded_name = domain.encode();
    /// 
    /// assert_eq!(encoded_name, b"\x03www\x07example\x03com\x00");
    /// ```
    pub fn encode(&self) -> Vec<u8> {
        let length = self.0.len() + self.0.iter().filter(|c| **c == 0x2e).count();
        let mut result = vec![0; length];

        let mut ptr = 0;
        for chunk in self.0.split(|c| *c == 0x2e) {
            result[ptr] = chunk.len() as u8;
            ptr += 1;
            result[ptr..ptr + chunk.len()].copy_from_slice(chunk);
            ptr += chunk.len();
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
    /// # use dns::message::Domain; 
    /// let encoded_name = b"\x03www\x07example\x03com\x00";
    /// let domain_name = Domain::decode(encoded_name);
    /// 
    /// assert_eq!(domain_name, b"www.example.com");
    /// ```
    /// 
    /// # Panics
    /// This function panics if the given byte buffer is not a valid encoded domain name,
    /// for example `\x03www\x07example\x04com`.
    pub fn decode(source: &[u8]) -> Vec<u8> {
        let mut result = vec![0; source.len() - 2];  // we dont need the null bytes & leading "."

        // creating a second view here simplifies the logic a bit because we don't need
        // to keep track of two indices
        let without_leading_dot = &source[1..];

        let mut block_size = source[0] as usize;
        let mut ptr = 0;
        
        loop {
            assert!(ptr + block_size < without_leading_dot.len(), "domain block reaches out of bounds");
            result[ptr..ptr + block_size].copy_from_slice(&without_leading_dot[ptr..ptr + block_size]);

            ptr += block_size;  

            block_size = without_leading_dot[ptr] as usize;
            if block_size == 0x00 {
                break;
            } else {
                result[ptr] = 0x2e;
                ptr += 1;
            }

        }

        result
    }
}

impl fmt::Debug for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Header")
         .field("id", &self.id)
         .field("num_questions", &self.num_questions)
         .field("num_answers", &self.num_answers)
         .field("num_authorities", &self.num_authorities)
         .field("num_additional", &self.num_additional)
         .field("response_code", &self.response_code())
         .field("recursion_available", &self.recursion_available())
         .field("recursion_desired", &self.recursion_desired())
         .field("truncated", &self.truncated())
         .field("authoritative", &self.authoritative())
         .finish()
    }
}