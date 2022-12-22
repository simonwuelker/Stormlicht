//! https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1

use random::RNG;

use std::{io::{Read, Write}, vec, fmt};

/// Encodes a domain name for use in a DNS query.
/// 
/// Blocks are seperated by a byte specifying their length.
/// The last byte is guaranteed to be a null byte
/// 
/// # Example
/// ```
/// # use dns::message::encode_domain_name; 
/// let domain_name = b"www.example.com";
/// let encoded_name = encode_domain_name(domain_name);
/// 
/// assert_eq!(encoded_name, b"\x03www\x07example\x03com\x00");
/// ```
pub fn encode_domain_name(domain_name: &[u8]) -> Vec<u8> {
    let length = domain_name.len() + domain_name.iter().filter(|c| **c == 0x2e).count();
    let mut result = vec![0; length];

    let mut ptr = 0;
    for chunk in domain_name.split(|c| *c == 0x2e) {
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
/// # use dns::message::decode_domain_name; 
/// let encoded_name = b"\x03www\x07example\x03com\x00";
/// let domain_name = decode_domain_name(encoded_name);
/// 
/// assert_eq!(domain_name, b"www.example.com");
/// ```
/// 
/// # Panics
/// This function panics if the given byte buffer is not a valid encoded domain name,
/// for example `\x03www\x07example\x04com`.
pub fn decode_domain_name(source: &[u8]) -> Vec<u8> {
    // NOTE: we overallocate, but just a bit (2-3 bytes most likely)
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

#[derive(Debug)]
pub enum QueryType {
    Standard = 0,
    Inverse = 1,
    Status = 2,
    Reserved,
}

pub enum ResponseCode {
    Ok = 0,
    FormatError = 1,
    ServerFailure = 2,
    NameError = 3,
    Unimplemented = 4,
    Refused = 5,
    Reserved
}

#[derive(Debug)]
// https://datatracker.ietf.org/doc/html/rfc1035#section-4.1
pub struct Message {
    header: Header,
    question: Vec<Question>,
    answer: Vec<Resource>,
    authority: Vec<Resource>,
    additional: Vec<Resource>,
}

#[derive(Debug)]
// https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1
pub struct Header {
    id: u16,
    flags: u16,
    num_questions: u16,
    num_answers: u16,
    num_ressources: u16,
    num_additional: u16,
}

// https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.2
pub struct Question {
    domain: Vec<u8>,
    query_type: QueryType,
    query_class: (),
}

#[derive(Debug)]
// https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.3
pub struct Resource {
    domain: Vec<u8>,
    resource_type: u16,
    class: u16,
    time_to_live: u32,
    rdlength: u16,
    data: Vec<u8>,
}

impl Header {
    pub fn new(num_questions: u16) -> Self {
        let id = RNG::default().next_u16();
        Self {
            id: RNG::default().next_u16(),
            flags: 0x100,
            num_questions: num_questions,
            num_answers: 0x0000,
            num_ressources: 0x0000,
            num_additional: 0x000,
        }
    }

    pub fn write_to_buffer(&self, bytes: &mut [u8]){
        bytes[0..2].copy_from_slice(&self.id.to_be_bytes());
        bytes[2..4].copy_from_slice(&self.flags.to_be_bytes());
        bytes[4..6].copy_from_slice(&self.num_questions.to_be_bytes());
        bytes[6..8].copy_from_slice(&self.num_answers.to_be_bytes());
        bytes[8..10].copy_from_slice(&self.num_ressources.to_be_bytes());
        bytes[10..12].copy_from_slice(&self.num_additional.to_be_bytes());
    }

    pub fn read(buffer: &[u8]) -> Result<Self, ()> {
        if buffer.len() < 12 {
            return Err(());
        }
        let id = u16::from_be_bytes(buffer[0..2].try_into().unwrap());
        let flags = u16::from_be_bytes(buffer[2..4].try_into().unwrap());
        let num_questions = u16::from_be_bytes(buffer[4..6].try_into().unwrap());
        let num_answers = u16::from_be_bytes(buffer[6..8].try_into().unwrap());
        let num_ressources = u16::from_be_bytes(buffer[8..10].try_into().unwrap());
        let num_additional = u16::from_be_bytes(buffer[10..12].try_into().unwrap());

        Ok(Self {
            id,
            flags,
            num_questions,
            num_answers,
            num_ressources,
            num_additional
        })

    }
}

impl Question {
    pub fn new(domain: Vec<u8>) -> Self {
        Self {
            domain: domain,
            query_type: QueryType::Standard,
            query_class: (),
        }
    }

    fn size(&self) -> usize {
        self.domain.len() + 4
    }

    pub fn write_to_buffer(&self, bytes: &mut [u8]) -> usize {
        let encoded_domain = encode_domain_name(&self.domain);
        bytes[..encoded_domain.len()].copy_from_slice(&encoded_domain);

        let mut ptr = encoded_domain.len();

        bytes[ptr..ptr + 2].copy_from_slice(&1_u16.to_be_bytes());
        ptr += 2;

        bytes[ptr..ptr + 2].copy_from_slice(&1_u16.to_be_bytes());
        ptr += 2;
        ptr
    }

    pub fn read(buffer: &[u8]) -> Result<(Self, usize), ()> {
        // Everything up to the first null byte (inclusive) is the encoded domain
        let domain_length = buffer.iter().position(|b| *b == 0x00).ok_or(())? + 1;
        let domain = decode_domain_name(&buffer[..domain_length]);

        let _query_type = u16::from_be_bytes(buffer[domain_length..domain_length + 2].try_into().unwrap());
        let _query_class = u16::from_be_bytes(buffer[domain_length + 2..domain_length + 4].try_into().unwrap());

        // FIXME properly parse the type/class
        Ok((Self {
            domain: domain,
            query_type: QueryType::Standard,
            query_class: (),
        }, domain_length +  4))
    }
}

impl Message {
    pub fn new(domain_name: &[u8]) -> Self {
        Self {
            header: Header::new(1),
            question: vec![Question::new(domain_name.to_vec())],
            answer: vec![],
            authority: vec![],
            additional: vec![],
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

    pub fn read(buffer: &[u8]) -> Result<Self, ()> {
        let header = Header::read(&buffer[..12])?;

        let mut questions = Vec::with_capacity(header.num_questions as usize);
        let mut answers = Vec::with_capacity(header.num_answers as usize);


        println!("{header:?}");

        let mut ptr = 12;
        for _ in 0..header.num_questions {
            let (new_question, bytes_read) = Question::read(&buffer[ptr..])?;
            questions.push(new_question);
            ptr += bytes_read;
        }

        println!("{questions:?}");



        Ok(Self {
            header: header,
            question: questions,
            answer: answers,
            authority: vec![],
            additional: vec![],
        })
    }
}

impl fmt::Debug for Question {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Question")
         .field("domain", &std::str::from_utf8(&self.domain).unwrap())
         .finish()
    }
}