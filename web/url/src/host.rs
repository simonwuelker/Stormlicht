use crate::{urlencode::percent_encode, urlparser::is_c0_control, util};

// https://url.spec.whatwg.org/#forbidden-host-code-point
fn is_forbidden_host_code_point(c: char) -> bool {
    match c {
        '\u{0000}' | '\u{0009}' | '\u{000A}' | '\u{000D}' | ' ' | '#' | '/' | ':' | '<' | '>'
        | '?' | '@' | '[' | '\\' | ']' | '^' | '|' => true,
        _ => false,
    }
}

// https://url.spec.whatwg.org/#forbidden-domain-code-point
fn is_forbidden_domain_code_point(c: char) -> bool {
    is_forbidden_host_code_point(c)
        | is_c0_control(c)
        | match c {
            '%' | '\u{007F}' => true,
            _ => false,
        }
}

// https://url.spec.whatwg.org/#ip-address
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum IP {
    IPv4(u32),
    IPv6([u16; 8]),
}

/// Typically either a network address or a opaque identifier in situations
/// where a network address is not required.
///
/// [Specification](https://url.spec.whatwg.org/#concept-host)
#[derive(PartialEq, Clone, Debug)]
pub enum Host {
    Domain(String),
    IP(IP),
    OpaqueHost(String),
    EmptyHost,
}

// https://url.spec.whatwg.org/#concept-host-parser
pub(crate) fn host_parse_with_special(input: &str, is_not_special: bool) -> Result<Host, ()> {
    // If input starts with U+005B ([), then:
    if input.starts_with('[') {
        // If input does not end with U+005D (])
        if !input.ends_with(']') {
            // validation error,
            // return failure.
            return Err(());
        }

        // Return the result of IPv6 parsing input with its leading U+005B ([) and trailing U+005D (]) removed.
        return Ok(Host::IP(IP::IPv6(ipv6_parse(&input[1..input.len() - 1])?)));
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
        return Err(());
    }

    // If asciiDomain ends in a number
    if ascii_domain.ends_with(|c: char| c.is_ascii_digit()) {
        // then return the result of IPv4 parsing asciiDomain.
        return Ok(Host::IP(IP::IPv4(ipv4_parse(input)?)));
    }

    // Return asciiDomain.
    Ok(Host::Domain(ascii_domain.to_string()))
}

// https://url.spec.whatwg.org/#concept-opaque-host-parser
fn opaque_host_parse(input: &str) -> Result<String, ()> {
    // If input contains a forbidden host code point
    if input.contains(is_forbidden_host_code_point) {
        // validation error, return failure.
        return Err(());
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

// https://url.spec.whatwg.org/#concept-ipv4-parser
fn ipv4_parse(input: &str) -> Result<u32, ()> {
    // Let validationError be false.
    let mut validation_error = false;

    // let parts be the result of strictly splitting input on U+002E (.)
    let mut parts: Vec<&str> = input.split('.').collect();

    // If the last item in parts is the empty string, then:
    if parts.last().is_none() || parts.last().as_ref().unwrap().is_empty() {
        // Set validationError to true.
        validation_error = true;

        // If parts’s size is greater than 1
        if parts.len() > 1 {
            // then remove the last item from parts.
            parts.pop();
        }
    }

    // If parts’s size is greater than 4
    if parts.len() > 4 {
        // validation error, return failure.
        return Err(());
    }

    // Let numbers be an empty list.
    let mut numbers = vec![];

    // For each part of parts:
    for part in parts {
        // Let result be the result of parsing part.
        let result = ipv4_number_parse(part);

        // If result is failure,
        if result.is_err() {
            // validation error, return failure.
            return Err(());
        }

        // If result[1] is true
        if result.unwrap().1 {
            // then set validationError to true.
            validation_error = true;
        }

        // Append result[0] to numbers.
        numbers.push(result.unwrap().0);
    }

    // If validationError is true,
    if validation_error {
        // validation error
    }

    // If any item in numbers is greater than 255,
    if numbers.iter().any(|n| *n > 255) {
        // validation error
    }

    // If any but the last item in numbers is greater than 255,
    if numbers[..numbers.len() - 1].iter().any(|n| *n > 255) {
        // then return failure.
        return Err(());
    }

    // If the last item in numbers is greater than or equal to 256^(5 − numbers’s size),
    if *numbers.last().unwrap() > 256_u32.pow(5 - numbers.len() as u32) {
        // validation error, return failure.
        return Err(());
    }

    // Let ipv4 be the last item in numbers.
    let mut ipv4: u32 = *numbers.last().unwrap();

    // Remove the last item from numbers.
    numbers.pop();

    // Let counter be 0.
    let mut counter = 0;

    // For each n of numbers:
    for n in numbers {
        // Increment ipv4 by n × 256^(3 − counter).
        ipv4 += n * 256_u32.pow(3 - counter);

        // Increment counter by 1.
        counter += 1;
    }

    // Return ipv4.
    Ok(ipv4)
}

// https://url.spec.whatwg.org/#ipv4-number-parser
fn ipv4_number_parse(mut input: &str) -> Result<(u32, bool), ()> {
    // If input is the empty string,
    if input.is_empty() {
        // then return failure
        return Err(());
    }

    // Let validationError be false.
    let mut validation_error = false;

    // Let R be 10.
    let mut radix = 10;

    // If input contains at least two code points
    // and the first two code points are either "0X" or "0x", then:
    if 2 <= input.len() && (input.starts_with("0x") || input.starts_with("0X")) {
        // Set validationError to true.
        validation_error = true;

        // Remove the first two code points from input.
        input = &input[2..];

        radix = 16;
    }
    // Otherwise, if input contains at least two code points
    // and the first code point is U+0030 (0), then:
    else if 2 <= input.len() && input.starts_with('0') {
        // Set validationError to true.
        validation_error = true;

        // Remove the first code point from input.
        input = &input[1..];

        // Set R to 8.
        radix = 8;
    }

    // If input is the empty string, then return (0, true).
    if input.is_empty() {
        return Ok((0, true));
    }

    // If input contains a code point that is not a radix-R digit,
    if input.contains(|c: char| c.is_digit(radix)) {
        // then return failure.
        return Err(());
    }

    // Let output be the mathematical integer value that is represented by input
    // in radix-R notation, using ASCII hex digits for digits with values 0 through 15.
    let output = u32::from_str_radix(input, radix).map_err(|_| ())?;

    // Return (output, validationError).
    Ok((output, validation_error))
}

// https://url.spec.whatwg.org/#concept-ipv6-parser
fn ipv6_parse(input: &str) -> Result<[u16; 8], ()> {
    // Let address be a new IPv6 address whose IPv6 pieces are all 0.
    let mut address = [0_u16; 8];

    // Let pieceIndex be 0.
    let mut piece_index = 0;

    // Let compress be null.
    let mut compress = None;

    // Let pointer be a pointer for input.
    let mut ptr = 0_usize;

    // If c is U+003A (:), then:
    if input.chars().nth(ptr) == Some(':') {
        // If remaining does not start with U+003A (:),
        if !input[ptr + 1..].starts_with(':') {
            // validation error, return failure.
            return Err(());
        }

        // Increase pointer by 2.
        ptr += 2;

        // Increase pieceIndex by 1
        piece_index += 1;

        // and then set compress to pieceIndex.
        compress = Some(piece_index);
    }

    // While c is not the EOF code point:
    while let Some(c) = input.chars().nth(ptr) {
        // If pieceIndex is 8,
        if piece_index == 8 {
            // validation error, return failure.
            return Err(());
        }

        // If c is U+003A (:), then:
        if c == ':' {
            // If compress is non-null,
            if compress.is_some() {
                // validation error, return failure.
                return Err(());
            }

            // Increase pointer and pieceIndex by 1,
            ptr += 1;
            piece_index += 1;

            // set compress to pieceIndex,
            compress = Some(piece_index);

            // and then continue.
            continue;
        }

        // Let value and length be 0.
        let mut value: u16 = 0;
        let mut length = 0;

        // While length is less than 4 and c is an ASCII hex digit
        while length < 4 && input.chars().nth(ptr).unwrap().is_ascii_hexdigit() {
            // set value to value × 0x10 + c interpreted as hexadecimal number
            value = value * 0x10 + input.chars().nth(ptr).unwrap().to_digit(16).unwrap() as u16;

            // and increase pointer and length by 1.
            ptr += 1;
            length += 1;
        }

        // If c is U+002E (.), then:
        if input.chars().nth(ptr) == Some('.') {
            // If length is 0
            if length == 0 {
                // validation error, return failure.
                return Err(());
            }

            // Decrease pointer by length.
            ptr -= length;

            // If pieceIndex is greater than 6
            if piece_index > 6 {
                // validation error, return failure.
                return Err(());
            }

            // Let numbersSeen be 0.
            let mut numbers_seen = 0;

            // While c is not the EOF code point:
            while input.chars().nth(ptr).is_some() {
                // Let ipv4Piece be null.
                let mut ipv4_piece: Option<u16> = None;

                // If numbersSeen is greater than 0, then:
                if numbers_seen > 0 {
                    // If c is a U+002E (.) and numbersSeen is less than 4,
                    if input.chars().nth(ptr).unwrap() == '.' && numbers_seen < 4 {
                        // then increase pointer by 1.
                        ptr += 1;
                    }
                    // Otherwise
                    else {
                        // validation error, return failure.
                        return Err(());
                    }
                }

                // If c is not an ASCII digit,
                if !input.chars().nth(ptr).unwrap().is_ascii_digit() {
                    // validation error, return failure.
                    return Err(());
                }

                // While c is an ASCII digit:
                while input.chars().nth(ptr).unwrap().is_ascii_digit() {
                    // Let number be c interpreted as decimal number.
                    let number = input.chars().nth(ptr).unwrap().to_digit(10).unwrap() as u16;

                    // If ipv4Piece is null,
                    if ipv4_piece.is_none() {
                        // then set ipv4Piece to number.
                        ipv4_piece = Some(number);
                    }
                    // Otherwise, if ipv4Piece is 0
                    else if ipv4_piece == Some(0) {
                        // validation error, return failure.
                        return Err(());
                    }
                    // Otherwise
                    else {
                        // set ipv4Piece to ipv4Piece × 10 + number.
                        ipv4_piece = Some(ipv4_piece.unwrap() * 10 + number);
                    }

                    // If ipv4Piece is greater than 255
                    if ipv4_piece.unwrap() > 255 {
                        // validation error, return failure.
                        return Err(());
                    }

                    // Increase pointer by 1.
                    ptr += 1;
                }

                // Set address[pieceIndex] to address[pieceIndex] × 0x100 + ipv4Piece.
                address[piece_index] = address[piece_index] * 0x100 + ipv4_piece.unwrap();

                // Increase numbersSeen by 1.
                numbers_seen += 1;

                // If numbersSeen is 2 or 4
                if numbers_seen == 2 || numbers_seen == 4 {
                    // then increase pieceIndex by 1.
                    piece_index += 1;
                }
            }

            // If numbersSeen is not 4
            if numbers_seen != 4 {
                // validation error, return failure.
                return Err(());
            }

            // Break.
            break;
        }
        // Otherwise, if c is U+003A (:):
        else if input.chars().nth(ptr) == Some(':') {
            // Increase pointer by 1.
            ptr += 1;

            // If c is the EOF code point,
            if input.chars().nth(ptr).is_none() {
                // validation error, return failure.
                return Err(());
            }
        }
        // Otherwise, if c is not the EOF code point
        else if !input.chars().nth(ptr).is_none() {
            // validation error, return failure.
            return Err(());
        }

        // Set address[pieceIndex] to value.
        address[piece_index] = value;

        // Increase pieceIndex by 1.
        piece_index += 1
    }

    // If compress is non-null, then:
    if let Some(compress_value) = compress {
        // Let swaps be pieceIndex − compress.
        let mut swaps = piece_index - compress_value;

        // Set pieceIndex to 7.
        piece_index = 7;

        // While pieceIndex is not 0 and swaps is greater than 0
        while piece_index != 0 && swaps > 0 {
            // swap address[pieceIndex] with address[compress + swaps − 1]
            let tmp = address[piece_index];
            address[piece_index] = address[compress_value + swaps - 1];
            address[compress_value + swaps - 1] = tmp;

            // and then decrease both pieceIndex and swaps by 1.
            piece_index -= 1;
            swaps -= 1;
        }
    }
    // Otherwise, if compress is null and pieceIndex is not 8
    else if piece_index != 8 {
        // validation error, return failure.
        return Err(());
    }

    // Return address.
    Ok(address)
}
