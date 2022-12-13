//! HTTP response parser

use nom::{
    bytes::streaming::{tag, take_while},
    character::{is_alphabetic, is_digit, streaming::space0},
    combinator::map_res,
    multi::many0,
    sequence::{delimited, separated_pair, terminated},
    IResult,
};
use std::collections::HashMap;

use status_code::StatusCode;

const NEWLINE: &[u8] = b"\r\n";

#[derive(Debug)]
pub struct Response {
    pub status: StatusCode,
    pub headers: HashMap<String, String>,
}

fn decimal(input: &[u8]) -> IResult<&[u8], usize> {
    map_res(take_while(is_digit), |out: &[u8]| {
        Ok::<usize, &[u8]>(
            out.iter()
                .map(|c| c - 0x30) // map to numeric value
                .reduce(|acc, x| (10 * acc) + x)
                .unwrap()
                .into(),
        )
    })(input)
}

fn is_field(c: u8) -> bool {
    // ascii printable, no space and no colon
    0x21 <= c && c <= 0x7e && c != 0x3a
}

pub fn parse_response(input: &[u8]) -> IResult<&[u8], Response> {
    let (input, _) = tag(b"HTTP/")(input)?;
    let (input, (major, minor)) = separated_pair(decimal, tag(b"."), decimal)(input)?;
    assert_eq!((major, minor), (1, 1), "unsupported http version");

    let (input, _) = tag(b" ")(input)?;
    let (input, response_code) = decimal(input)?;
    let (input, _) = tag(b" ")(input)?;
    let (input, _) = terminated(take_while(is_alphabetic), tag(NEWLINE))(input)?; // response text SHOULD not be relied upon

    let (input, headers) = terminated(
        many0(terminated(
            separated_pair(
                take_while(is_field), // field name
                tag(b":"),
                delimited(
                    space0,
                    take_while(is_field), // field value
                    space0,
                ),
            ),
            tag(NEWLINE),
        )),
        tag(NEWLINE),
    )(input)?;

    let mut header_map = HashMap::new();
    for (key, value) in headers {
        header_map.insert(
            std::str::from_utf8(key).unwrap().to_owned(),
            std::str::from_utf8(value).unwrap().to_owned(),
        );
    }

    Ok((
        input,
        Response {
            status: StatusCode::try_from(response_code).unwrap(),
            headers: header_map,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_response() {
        let response_raw = b"HTTP/1.1 200 OK\r\n\
        host: google\r\n\
        \r\n";
        let (_, response) = parse_response(response_raw).unwrap();

        assert_eq!(response.status, StatusCode::Ok);
        assert_eq!(response.headers["host"], "google");
    }
}
