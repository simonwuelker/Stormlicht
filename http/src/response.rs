//! HTTP/1.1 response parser

use parser_combinators::{literal, predicate, some, ParseResult, Parser, ParserCombinator};
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
}
