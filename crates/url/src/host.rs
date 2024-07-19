use std::{fmt, net};

use sl_std::{ascii, punycode};

use crate::{
    ip::{ipv4_parse, ipv6_parse},
    percent_encode::{is_c0_control, percent_encode, C0_CONTROL},
    AsciiSet, IPParseError,
};

/// <https://url.spec.whatwg.org/#forbidden-host-code-point>
const FORBIDDEN_HOST_CODE_POINTS: AsciiSet = AsciiSet::EMPTY
    .add(ascii::Char::Null)
    .add(ascii::Char::CharacterTabulation)
    .add(ascii::Char::LineFeed)
    .add(ascii::Char::CarriageReturn)
    .add(ascii::Char::Space)
    .add(ascii::Char::NumberSign)
    .add(ascii::Char::Solidus)
    .add(ascii::Char::Colon)
    .add(ascii::Char::GreaterThanSign)
    .add(ascii::Char::LessThanSign)
    .add(ascii::Char::QuestionMark)
    .add(ascii::Char::CommercialAt)
    .add(ascii::Char::LeftSquareBracket)
    .add(ascii::Char::ReverseSolidus)
    .add(ascii::Char::RightSquareBracket)
    .add(ascii::Char::CircumflexAccent)
    .add(ascii::Char::VerticalLine);

/// <https://url.spec.whatwg.org/#forbidden-domain-code-point>
const FORBIDDEN_DOMAIN_CODE_POINTS: AsciiSet = FORBIDDEN_HOST_CODE_POINTS
    .merge(C0_CONTROL)
    .add(ascii::Char::PercentSign)
    .add(ascii::Char::Delete);

/// Typically either a network address or a opaque identifier in situations
/// where a network address is not required.
///
/// [Specification](https://url.spec.whatwg.org/#concept-host)
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialize",
    derive(serialize::Serialize, serialize::Deserialize)
)]
pub enum Host {
    Domain(ascii::String),
    Ip(net::IpAddr),
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

impl fmt::Display for Host {
    // <https://url.spec.whatwg.org/#host-serializing>
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ip(net::IpAddr::V4(ipv4)) => {
                // 1. If host is an IPv4 address, return the result of running the IPv4 serializer on host.
                ipv4.fmt(f)
            },
            Self::Ip(net::IpAddr::V6(ipv6)) => {
                // 2. Otherwise, if host is an IPv6 address, return U+005B ([), followed by the result of running the IPv6 serializer on host, followed by U+005D (]).
                write!(f, "[{ipv6}]")
            },
            Self::Domain(host) | Self::OpaqueHost(host) => {
                // 3. Otherwise, host is a domain, opaque host, or empty host, return host.
                host.as_str().fmt(f)
            },
            Self::EmptyHost => {
                // 3. Otherwise, host is a domain, opaque host, or empty host, return host.
                Ok(())
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
            // return failure.
            return Err(HostParseError::MalformedInput);
        }

        // Return the result of IPv6 parsing input with its leading U+005B ([) and trailing U+005D (]) removed.
        let ipv6_text = &input[1..input.len() - 1];
        let ipv6 = ipv6_parse(ipv6_text).map_err(HostParseError::IP)?;
        let host = Host::Ip(net::IpAddr::V6(ipv6));
        return Ok(host);
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
        .any(|c| FORBIDDEN_DOMAIN_CODE_POINTS.contains(c))
    {
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
        let ipv4 = ipv4_parse(input).map_err(HostParseError::IP)?;
        return Ok(Host::Ip(net::IpAddr::V4(ipv4)));
    }

    // Return asciiDomain.
    Ok(Host::Domain(ascii_domain))
}

/// <https://url.spec.whatwg.org/#concept-opaque-host-parser>
fn opaque_host_parse(input: &str) -> Result<ascii::String, HostParseError> {
    // If input contains a forbidden host code point
    let has_forbidden_host_code_point = input.contains(|c: char| {
        c.as_ascii()
            .is_some_and(|c| FORBIDDEN_HOST_CODE_POINTS.contains(c))
    });

    if has_forbidden_host_code_point {
        // return failure.
        return Err(HostParseError::ForbiddenCodePoint);
    }

    // FIXME: If input contains a U+0025 (%) and the two code points
    // following it are not ASCII hex digits, invalid-URL-unit validation error.

    // Return the result of running UTF-8 percent-encode on input
    // using the C0 control percent-encode set.
    let mut percent_encoded = ascii::String::with_capacity(input.len());
    percent_encode(input.as_bytes(), is_c0_control, &mut percent_encoded);
    Ok(percent_encoded)
}
