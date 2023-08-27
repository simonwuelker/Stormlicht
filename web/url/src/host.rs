use std::str::FromStr;

use crate::{
    parser::is_c0_control, urlencode::percent_encode, util, IPParseError, Ipv4Address, Ipv6Address,
};

/// <https://url.spec.whatwg.org/#forbidden-host-code-point>
fn is_forbidden_host_code_point(c: char) -> bool {
    matches!(
        c,
        '\u{0000}'
            | '\u{0009}'
            | '\u{000A}'
            | '\u{000D}'
            | ' '
            | '#'
            | '/'
            | ':'
            | '<'
            | '>'
            | '?'
            | '@'
            | '['
            | '\\'
            | ']'
            | '^'
            | '|'
    )
}

/// <https://url.spec.whatwg.org/#forbidden-domain-code-point>
fn is_forbidden_domain_code_point(c: char) -> bool {
    is_forbidden_host_code_point(c) | is_c0_control(c) | matches!(c, '%' | '\u{007F}')
}

/// Typically either a network address or a opaque identifier in situations
/// where a network address is not required.
///
/// [Specification](https://url.spec.whatwg.org/#concept-host)
#[derive(PartialEq, Clone, Debug)]
pub enum Host {
    Domain(String),
    IPv4(Ipv4Address),
    IPv6(Ipv6Address),
    OpaqueHost(String),
    EmptyHost,
}

#[derive(Clone, Copy, Debug)]
pub enum HostParseError {
    MalformedInput,
    ForbiddenCodePoint,
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
                host.clone()
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
    // (Since we don't support any encodings other than utf-8 this is not necessary)

    // If asciiDomain is failure, validation error, return failure.
    // TODO

    let ascii_domain = input;
    // If asciiDomain contains a forbidden domain code point,
    if ascii_domain.contains(is_forbidden_domain_code_point) {
        // validation error,
        // return failure.
        return Err(HostParseError::ForbiddenCodePoint);
    }

    // If asciiDomain ends in a number
    if ascii_domain.ends_with(|c: char| c.is_ascii_digit()) {
        // then return the result of IPv4 parsing asciiDomain.
        return Ok(Host::IPv4(
            Ipv4Address::from_str(input).map_err(HostParseError::IP)?,
        ));
    }

    // Return asciiDomain.
    Ok(Host::Domain(ascii_domain.to_string()))
}

/// <https://url.spec.whatwg.org/#concept-opaque-host-parser>
fn opaque_host_parse(input: &str) -> Result<String, HostParseError> {
    // If input contains a forbidden host code point
    if input.contains(is_forbidden_host_code_point) {
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
    Ok(percent_encode(input, is_c0_control))
}
