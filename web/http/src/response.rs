//! HTTP/1.1 response parser

use parser_combinators::{
    literal, optional, predicate, some, ParseResult, Parser, ParserCombinator,
};
use std::collections::HashMap;

use crate::status_code::StatusCode;

#[derive(Debug)]
pub struct Response {
    pub status: StatusCode,
    pub headers: HashMap<String, String>,
}

pub fn parse_status_line<'a>(input: &'a [u8]) -> ParseResult<&'a [u8], u32> {
    let http_version = literal(b"HTTP/1.1");
    let whitespace = literal(b" ");
    let linebreak = literal(b"\r\n");
    let digit = predicate(|input: &[u8]| {
        if input.len() == 0 {
            Err(0)
        } else {
            let maybe_char = input[0];
            if 0x30 <= maybe_char && maybe_char < 0x3A {
                Ok((&input[1..], maybe_char - 0x30))
            } else {
                Err(0)
            }
        }
    });
    let character = predicate(|input: &[u8]| {
        if input.len() == 0 {
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
    let parser = http_version
        .then(whitespace)
        .then(status_code)
        .map(|res| res.1)
        .then(whitespace)
        .map(|res| res.0)
        .then(some(character))
        .map(|res| res.0)
        .then(linebreak)
        .map(|res| res.0);
    parser.parse(input)
}

pub fn parse_header<'a>(input: &'a [u8]) -> ParseResult<&'a [u8], (String, String)> {
    let legal_name_chars = predicate(|input: &[u8]| {
        if input.len() == 0 {
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
        if input.len() == 0 {
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
    let to_string = |chars: Vec<u8>| {
        chars
            .iter()
            .map(|byte| char::from_u32(*byte as u32).unwrap())
            .collect::<String>()
            .trim()
            .to_string()
    };
    let colon = literal(b":");
    let whitespace = literal(b" ");
    some(legal_name_chars)
        .map(to_string)
        .then(colon)
        .map(|res| res.0)
        .then(optional(whitespace))
        .map(|res| res.0)
        .then(some(legal_value_chars))
        .map(|(field, value_bytes)| (field, to_string(value_bytes)))
        .then(linebreak)
        .map(|res| res.0)
        .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_status_line() {
        let response_raw = b"HTTP/1.1 200 OK\r\n";
        let parse_result = parse_status_line(response_raw);
        assert!(parse_result.is_ok());

        let (_, response_code) = parse_result.unwrap();
        assert_eq!(response_code, 200);
    }

    #[test]
    fn test_parse_header() {
        let header_raw = b"User-Agent: curl/7.64.1\r\n";
        let parse_result = parse_header(header_raw);
        assert!(parse_result.is_ok());

        let (_, header) = parse_result.unwrap();
        assert_eq!(
            header,
            ("User-Agent".to_string(), "curl/7.64.1".to_string())
        );
    }
}
