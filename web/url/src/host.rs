use std::str::FromStr;

use sl_std::{ascii, punycode};

use crate::{
    urlencode::{is_c0_control, percent_encode},
    util, IPParseError, Ipv4Address, Ipv6Address,
};

/// <https://url.spec.whatwg.org/#forbidden-host-code-point>
fn is_forbidden_host_code_point(c: ascii::Char) -> bool {
    matches!(
        c,
        ascii::Char::Null
            | ascii::Char::CharacterTabulation
            | ascii::Char::LineFeed
            | ascii::Char::CarriageReturn
            | ascii::Char::Space
            | ascii::Char::NumberSign
            | ascii::Char::Solidus
            | ascii::Char::Colon
            | ascii::Char::GreaterThanSign
            | ascii::Char::LessThanSign
            | ascii::Char::QuestionMark
            | ascii::Char::CommercialAt
            | ascii::Char::LeftSquareBracket
            | ascii::Char::ReverseSolidus
            | ascii::Char::RightSquareBracket
            | ascii::Char::CircumflexAccent
            | ascii::Char::VerticalLine
    )
}

/// <https://url.spec.whatwg.org/#forbidden-domain-code-point>
fn is_forbidden_domain_code_point(c: ascii::Char) -> bool {
    is_forbidden_host_code_point(c)
        | is_c0_control(c as u8)
        | matches!(c, ascii::Char::PercentSign | ascii::Char::Delete)
}

/// Typically either a network address or a opaque identifier in situations
/// where a network address is not required.
///
/// [Specification](https://url.spec.whatwg.org/#concept-host)
#[derive(PartialEq, Clone, Debug)]
pub enum Host {
    Domain(ascii::String),
    IPv4(Ipv4Address),
    IPv6(Ipv6Address),
    OpaqueHost(ascii::String),
    EmptyHost,
}

#[derive(Clone, Copy, Debug)]
pub enum HostParseError {
    MalformedInput,
    ForbiddenCodePoint,
    Punycode(punycode::PunyCodeError),
    IP(IPParseError),
}

impl ToString for Host {
    // <https://url.spec.whatwg.org/#host-serializing>
    fn to_string(&self) -> String {
        match self {
            Self::IPv4(ipv4) => {
                // 1. If host is an IPv4 address, return the result of running the IPv4 serializer on host.
                ipv4.to_string()
            },
            Self::IPv6(ipv6) => {
                // 2. Otherwise, if host is an IPv6 address, return U+005B ([), followed by the result of running the IPv6 serializer on host, followed by U+005D (]).
                format!("[{}]", ipv6.to_string())
            },
            Self::Domain(host) | Self::OpaqueHost(host) => {
                // 3. Otherwise, host is a domain, opaque host, or empty host, return host.
                host.as_str().to_owned()
            },
            Self::EmptyHost => {
                // 3. Otherwise, host is a domain, opaque host, or empty host, return host.
                String::new()
            },
        }
    }
}

/// <https://url.spec.whatwg.org/#concept-host-parser>
pub(crate) fn host_parse_with_special(
    input: &str,
    is_not_special: bool,
) -> Result<Host, HostParseError> {
    // If input starts with U+005B ([), then:
    if input.starts_with('[') {
        // If input does not end with U+005D (])
        if !input.ends_with(']') {
            // validation error,
            // return failure.
            return Err(HostParseError::MalformedInput);
        }

        // Return the result of IPv6 parsing input with its leading U+005B ([) and trailing U+005D (]) removed.
        let ipv6_text = &input[1..input.len() - 1];
        let parsed_ip = Host::IPv6(Ipv6Address::from_str(ipv6_text).map_err(HostParseError::IP)?);
        return Ok(parsed_ip);
    }

    // If isNotSpecial is true
    if is_not_special {
        // then return the result of opaque-host parsing input.
        return Ok(Host::OpaqueHost(opaque_host_parse(input)?));
    }

    // Let domain be the result of running UTF-8 decode without BOM on the percent-decoding of input.

    // Assert: input is not the empty string.
    assert!(!input.is_empty());

    // Let domain be the result of running
    // UTF-8 decode without BOM on the percent-decoding of input.
    // TODO

    // Let asciiDomain be the result of running domain to ASCII with domain and false.
    // If asciiDomain is failure, validation error, return failure.
    let ascii_domain =
        ascii::String::from_utf8_punycode(input).map_err(HostParseError::Punycode)?;

    // If asciiDomain contains a forbidden domain code point,
    if ascii_domain
        .chars()
        .iter()
        .copied()
        .any(is_forbidden_domain_code_point)
    {
        // validation error,
        // return failure.
        return Err(HostParseError::ForbiddenCodePoint);
    }

    // If asciiDomain ends in a number
    if ascii_domain
        .chars()
        .last()
        .is_some_and(|&c| ascii::Char::Digit0 <= c && c <= ascii::Char::Digit9)
    {
        // then return the result of IPv4 parsing asciiDomain.
        return Ok(Host::IPv4(
            Ipv4Address::from_str(input).map_err(HostParseError::IP)?,
        ));
    }

    // Return asciiDomain.
    Ok(Host::Domain(ascii_domain))
}

/// <https://url.spec.whatwg.org/#concept-opaque-host-parser>
fn opaque_host_parse(input: &str) -> Result<ascii::String, HostParseError> {
    // If input contains a forbidden host code point
    if input.contains(|c: char| c.as_ascii().is_some_and(is_forbidden_host_code_point)) {
        // validation error, return failure.
        return Err(HostParseError::ForbiddenCodePoint);
    }

    // If input contains a code point that is not a URL code point and not U+0025 (%)
    if input.contains(|c| !util::is_url_codepoint(c) && c != '%') {
        // validation error
    }

    // If input contains a U+0025 (%) and the two code points
    // following it are not ASCII hex digits, validation error.
    // TODO

    // Return the result of running UTF-8 percent-encode on input
    // using the C0 control percent-encode set.
    let mut percent_encoded = ascii::String::with_capacity(input.len());
    percent_encode(input, is_c0_control, &mut percent_encoded);
    Ok(percent_encoded)
}
