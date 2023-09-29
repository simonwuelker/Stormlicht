//! https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1

use sl_std::{rand::RNG, read::ReadExt};

use crate::{domain::Domain, ResourceRecordClass, ResourceRecordType};
use std::{fmt, io, net::IpAddr, vec};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueryType {
    Standard,
    Inverse,
    Status,
    Reserved,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct Flags(u16);

/// <https://datatracker.ietf.org/doc/html/rfc1035#section-4.1>
#[derive(Clone, Debug)]
pub(crate) struct Message {
    header: Header,
    question: Vec<Question>,
    answer: Vec<Resource>,
    authority: Vec<Resource>,
    additional: Vec<Resource>,
}

/// <https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1>
#[derive(Clone, Copy)]
pub struct Header {
    id: u16,
    flags: Flags,
    num_questions: u16,
    num_answers: u16,
    num_authorities: u16,
    num_additional: u16,
}

/// <https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.2>
#[derive(Clone, Debug)]
pub struct Question {
    domain: Domain,
    _query_type: QueryType,
    _query_class: (),
}

/// <https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.3>
#[derive(Clone, Debug)]
pub struct Resource {
    pub domain: Domain,
    pub resource_type: ResourceRecordType,
    pub class: ResourceRecordClass,
    pub time_to_live: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    #[must_use]
    pub fn new(num_questions: u16) -> Self {
        Self {
            id: RNG::default().next_u16(),
            flags: Flags::default(),
            num_questions: num_questions,
            num_answers: 0x0000,
            num_authorities: 0x0000,
            num_additional: 0x000,
        }
    }

    pub fn write_to_buffer(&self, bytes: &mut [u8]) {
        bytes[0..2].copy_from_slice(&self.id.to_be_bytes());
        bytes[2..4].copy_from_slice(&self.flags.0.to_be_bytes());
        bytes[4..6].copy_from_slice(&self.num_questions.to_be_bytes());
        bytes[6..8].copy_from_slice(&self.num_answers.to_be_bytes());
        bytes[8..10].copy_from_slice(&self.num_authorities.to_be_bytes());
        bytes[10..12].copy_from_slice(&self.num_additional.to_be_bytes());
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

    #[inline]
    #[must_use]
    pub fn id(&self) -> u16 {
        self.header.id
    }

    #[must_use]
    pub fn size(&self) -> usize {
        16 + self.question.iter().map(|q| q.size()).sum::<usize>()
    }

    #[inline]
    #[must_use]
    pub fn additional_records(&self) -> &[Resource] {
        &self.additional
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

        let mut cursor = io::Cursor::new(buffer);

        // FIXME: propagate errors
        let id = cursor.read_be_u16().map_err(|_| ())?;
        let flags = Flags::new(cursor.read_be_u16().map_err(|_| ())?);
        let num_questions = cursor.read_be_u16().map_err(|_| ())?;
        let num_answers = cursor.read_be_u16().map_err(|_| ())?;
        let num_authorities = cursor.read_be_u16().map_err(|_| ())?;
        let num_additional = cursor.read_be_u16().map_err(|_| ())?;

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

        let mut cursor = io::Cursor::new(&buffer[domain_length + 2..]);
        let class = cursor.read_be_u16().map_err(|_| ())?.into();
        let ttl = cursor.read_be_u32().map_err(|_| ())?;
        let rdlength = cursor.read_be_u16().map_err(|_| ())? as usize;

        Ok((
            Self {
                domain: domain,
                resource_type: rtype,
                class,
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
            .field("flags", &self.flags)
            .finish()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageType {
    Question,
    Response,
}

impl Flags {
    #[inline]
    #[must_use]
    pub const fn new(value: u16) -> Self {
        debug_assert!(
            value & 0b1110000 == 0,
            "reserved Z field used in DNS header"
        );

        Self(value)
    }

    #[inline]
    #[must_use]
    pub const fn message_type(&self) -> MessageType {
        if self.0 & 0x8000 == 0 {
            MessageType::Question
        } else {
            MessageType::Response
        }
    }

    #[inline]
    #[must_use]
    pub const fn set_message_type(mut self, message_type: MessageType) -> Self {
        match message_type {
            MessageType::Question => self.0 &= !0x8000,
            MessageType::Response => self.0 |= 0x8000,
        }
        self
    }

    #[inline]
    #[must_use]
    pub const fn query_type(&self) -> QueryType {
        match (self.0 & 0x7800) >> 4 {
            0 => QueryType::Standard,
            1 => QueryType::Inverse,
            2 => QueryType::Status,
            3.. => QueryType::Reserved,
        }
    }

    #[inline]
    #[must_use]
    pub const fn set_query_type(mut self, query_type: QueryType) -> Self {
        self.0 &= !0x7800;

        match query_type {
            QueryType::Standard => {},
            QueryType::Inverse => self.0 |= 0x800,
            QueryType::Status => self.0 |= 0x1000,
            QueryType::Reserved => self.0 |= 0x1800,
        }
        self
    }

    #[inline]
    #[must_use]
    pub const fn is_authoritative(&self) -> bool {
        self.0 & 0x400 != 0
    }

    #[inline]
    #[must_use]
    pub const fn set_is_authoritative(mut self, is_authoritative: bool) -> Self {
        if is_authoritative {
            self.0 |= 0x400;
        } else {
            self.0 &= !0x400;
        }

        self
    }

    #[inline]
    #[must_use]
    pub const fn is_truncated(&self) -> bool {
        self.0 & 0x200 != 0
    }

    #[inline]
    #[must_use]
    pub const fn set_is_truncated(mut self, is_truncated: bool) -> Self {
        if is_truncated {
            self.0 |= 0x200;
        } else {
            self.0 &= !0x200;
        }

        self
    }

    #[inline]
    #[must_use]
    pub const fn recursion_desired(&self) -> bool {
        self.0 & 0x100 != 0
    }

    #[inline]
    #[must_use]
    pub const fn set_recursion_desired(mut self, recursion_desired: bool) -> Self {
        if recursion_desired {
            self.0 |= 0x100;
        } else {
            self.0 &= !0x100;
        }

        self
    }

    #[inline]
    #[must_use]
    pub const fn recursion_available(&self) -> bool {
        self.0 & 0x80 != 0
    }

    #[inline]
    #[must_use]
    pub const fn set_recursion_available(mut self, recursion_desired: bool) -> Self {
        if recursion_desired {
            self.0 |= 0x80;
        } else {
            self.0 &= !0x80;
        }

        self
    }

    #[inline]
    #[must_use]
    pub const fn response_code(&self) -> ResponseCode {
        match self.0 & 0b1111 {
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

impl fmt::Debug for Flags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Flags")
            .field("message_type", &self.message_type())
            .field("opcode", &self.query_type())
            .field("is_authoritative", &self.is_authoritative())
            .field("is_truncated", &self.is_truncated())
            .field("recursion_desired", &self.recursion_desired())
            .field("recursion_available", &self.recursion_available())
            .field("response_code", &self.response_code())
            .finish()
    }
}
