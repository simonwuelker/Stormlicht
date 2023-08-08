//! https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1
use random::RNG;

use crate::{domain::Domain, ResourceRecordClass, ResourceRecordType};
use std::{fmt, net::IpAddr, vec};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    pub(crate) authority: Vec<Resource>,
    pub(crate) additional: Vec<Resource>,
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
    time_to_live: u32,
}

#[derive(Debug)]
pub enum ResponseCode {
    /// No error condition
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

    pub fn write_to_buffer(&self, bytes: &mut [u8]) {
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
    #[must_use]
    pub fn new(domain: Domain) -> Self {
        Self {
            domain: domain,
            _query_type: QueryType::Standard,
            _query_class: (),
        }
    }

    #[must_use]
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
    #[must_use]
    pub fn new(domain: &Domain) -> Self {
        Self {
            header: Header::new(1),
            question: vec![Question::new(domain.clone())],
            answer: vec![],
            authority: vec![],
            additional: vec![],
        }
    }

    #[must_use]
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

    /// Returns a tuple `(resolved IP, TTL)`
    pub fn get_answer(&self, domain: &Domain) -> Option<(IpAddr, u32)> {
        for answer in self.answer.iter().chain(&self.additional) {
            if answer.domain == *domain {
                match answer.resource_type {
                    ResourceRecordType::A { ipv4 } => {
                        return Some((IpAddr::V4(ipv4), answer.time_to_live))
                    },
                    ResourceRecordType::CNAME { ref alias } => {
                        return self.get_answer(alias);
                    },
                    _ => {},
                }
            }
        }

        // Our question was not answered
        None
    }

    pub fn get_authority(&self, _domain: &Domain) -> Option<Domain> {
        for authority in &self.authority {
            if let ResourceRecordType::NS { ns } = &authority.resource_type {
                return Some(ns.clone());
            }
        }

        // No authoritative nameserver was provided
        None
    }
}

/// Generic trait for DNS-related things that can be read from a
/// byte buffer
pub(crate) trait Consume {
    /// Tries to consume the given data structure from a buffer at a given offset.
    /// Fails if the buffer is too small or the data is invalid.
    /// On success, returns the parsed data structure and the number of bytes (from ptr)
    /// that were read
    fn read(buffer: &[u8], ptr: usize) -> Result<(Self, usize), ()>
    where
        Self: Sized;
}

impl Consume for Message {
    fn read(buffer: &[u8], mut ptr: usize) -> Result<(Self, usize), ()> {
        let (header, bytes_read) = Header::read(buffer, ptr)?;
        ptr += bytes_read;

        let mut questions = Vec::with_capacity(header.num_questions as usize);
        let mut answers = Vec::with_capacity(header.num_answers as usize);
        let mut authority = Vec::with_capacity(header.num_authorities as usize);
        let mut additional = Vec::with_capacity(header.num_additional as usize);

        for _ in 0..header.num_questions {
            let (new_question, bytes_read) = Question::read(buffer, ptr)?;
            questions.push(new_question);
            ptr += bytes_read;
        }

        for _ in 0..header.num_answers {
            let (new_answer, bytes_read) = Resource::read(buffer, ptr)?;
            answers.push(new_answer);
            ptr += bytes_read;
        }

        for _ in 0..header.num_authorities {
            let (new_authority, bytes_read) = Resource::read(buffer, ptr)?;
            authority.push(new_authority);
            ptr += bytes_read;
        }

        for _ in 0..header.num_additional {
            let (new_additional, bytes_read) = Resource::read(buffer, ptr)?;
            additional.push(new_additional);
            ptr += bytes_read;
        }

        Ok((
            Self {
                header: header,
                question: questions,
                answer: answers,
                authority: authority,
                additional: additional,
            },
            ptr,
        ))
    }
}

impl Consume for Question {
    fn read(global_buffer: &[u8], ptr: usize) -> Result<(Self, usize), ()> {
        let buffer = &global_buffer[ptr..];

        let (domain, domain_length) = Domain::read(global_buffer, ptr)?;

        let _query_type =
            u16::from_be_bytes(buffer[domain_length..domain_length + 2].try_into().unwrap());
        let _query_class = u16::from_be_bytes(
            buffer[domain_length + 2..domain_length + 4]
                .try_into()
                .unwrap(),
        );

        // FIXME properly parse the type/class
        Ok((
            Self {
                domain: domain,
                _query_type: QueryType::Standard,
                _query_class: (),
            },
            domain_length + 4,
        ))
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

        Ok((
            Self {
                id,
                flags,
                num_questions,
                num_answers,
                num_authorities,
                num_additional,
            },
            12,
        ))
    }
}

impl Consume for Resource {
    fn read(global_buffer: &[u8], ptr: usize) -> Result<(Self, usize), ()> {
        let buffer = &global_buffer[ptr..];
        let (domain, domain_length) = Domain::read(global_buffer, ptr)?;

        let rtype = (global_buffer, ptr + domain_length).try_into()?;

        let class = u16::from_be_bytes(
            buffer[domain_length + 2..domain_length + 4]
                .try_into()
                .unwrap(),
        )
        .into();
        let ttl = u32::from_be_bytes(
            buffer[domain_length + 4..domain_length + 8]
                .try_into()
                .unwrap(),
        );
        let rdlength = u16::from_be_bytes(
            buffer[domain_length + 8..domain_length + 10]
                .try_into()
                .unwrap(),
        ) as usize;

        Ok((
            Self {
                domain: domain,
                resource_type: rtype,
                _class: class,
                time_to_live: ttl,
            },
            domain_length + 10 + rdlength,
        ))
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
