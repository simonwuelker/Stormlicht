//! HTTP/1.1 response parser

use std::collections::HashMap;

use parser_combinators::{
    literal, many, optional, predicate, some, ParseResult, Parser, ParserCombinator,
};

use crate::status_code::StatusCode;

#[derive(Clone, Debug)]
pub struct Response {
    pub status: StatusCode,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Response {
    pub fn get_header(&self, header: &str) -> Option<&str> {
        for (key, value) in &self.headers {
            if key.eq_ignore_ascii_case(header) {
                return Some(value);
            }
        }
        None
    }
}

pub(crate) fn parse_response(input: &[u8]) -> ParseResult<&[u8], Response> {
    let http_version = literal(b"HTTP/1.1");
    let whitespace = literal(b" ");
    let linebreak = literal(b"\r\n");
    let digit = predicate(|input: &[u8]| {
        if input.is_empty() {
            Err(0)
        } else {
            let maybe_char = input[0];
            if (0x30..0x3A).contains(&maybe_char) {
                Ok((&input[1..], maybe_char - 0x30))
            } else {
                Err(0)
            }
        }
    });
    let character = predicate(|input: &[u8]| {
        if input.is_empty() {
            Err(0)
        } else {
            match input[0] {
                0x20 | 0x41..=0x5A | 0x61..=0x7A => Ok((&input[1..], ())),
                _ => Err(0),
            }
        }
    });

    let status_code =
        some(digit).map(|digits| digits.iter().fold(0_u32, |acc, x| 10 * acc + *x as u32));

    let status_line = http_version
        .then(whitespace)
        .then(status_code)
        .map(|res| res.1)
        .then(whitespace)
        .map(|res| res.0)
        .then(some(character))
        .map(|res| res.0)
        .then(linebreak)
        .map(|res| res.0);

    let legal_name_chars = predicate(|input: &[u8]| {
        if input.is_empty() {
            Err(0)
        } else {
            match input[0] {
                // ascii printable without colon
                0x20..=0x39 | 0x3B..=0x7E => Ok((&input[1..], input[0])),
                _ => Err(0),
            }
        }
    });
    let legal_value_chars = predicate(|input: &[u8]| {
        if input.is_empty() {
            Err(0)
        } else {
            match input[0] {
                // ascii printable
                0x20..=0x7E => Ok((&input[1..], input[0])),
                _ => Err(0),
            }
        }
    });
    let linebreak = literal(b"\r\n");
    let to_string = |bytes: Vec<u8>| {
        bytes
            .iter()
            .map(|byte| char::from_u32(*byte as u32).unwrap())
            .collect::<String>()
            .trim()
            .to_string()
    };
    let colon = literal(b":");
    let whitespace = literal(b" ");
    let headers = many(
        some(legal_name_chars)
            .map(to_string)
            .then(colon)
            .map(|res| res.0)
            .then(optional(whitespace))
            .map(|res| res.0)
            .then(some(legal_value_chars))
            .map(|(field, value_bytes)| (field, to_string(value_bytes)))
            .then(linebreak)
            .map(|res| res.0),
    )
    .map(|header_list| {
        let mut headers = HashMap::with_capacity(header_list.len());
        for (key, value) in header_list {
            headers.insert(key, value);
        }
        headers
    });

    status_line
        .then(headers)
        .map(|(response_code, headers)| Response {
            status: StatusCode::try_from(response_code).unwrap(),
            headers: headers,
            body: vec![],
        })
        .parse(input)
}
