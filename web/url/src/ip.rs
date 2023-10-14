use std::net;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IPParseError {
    /// A value exceeded the maximum of `255`
    Ipv4NumberTooLarge,
    InvalidLastNumber,

    /// <https://url.spec.whatwg.org/#ipv4-in-ipv6-too-many-pieces>
    IPv4InIpv6TooManyParts,

    /// <https://url.spec.whatwg.org/#ipv4-non-numeric-part>
    Ipv4NonNumericPart,

    /// <https://url.spec.whatwg.org/#ipv4-too-many-parts>
    Ipv4TooManyParts,

    /// <https://url.spec.whatwg.org/#ipv6-invalid-compression>
    Ipv6InvalidCompression,

    /// <https://url.spec.whatwg.org/#ipv6-too-many-pieces>
    Ipv6TooManyPieces,

    /// <https://url.spec.whatwg.org/#ipv6-multiple-compression>
    Ipv6MultipleCompression,

    /// <https://url.spec.whatwg.org/#ipv4-in-ipv6-invalid-code-point>
    Ipv4InIpv6InvalidCodepoint,

    /// <https://url.spec.whatwg.org/#ipv4-in-ipv6-out-of-range-part>
    Ipv4InIpv6OutOfRangePart,

    /// <https://url.spec.whatwg.org/#ipv4-in-ipv6-too-few-parts>
    Ipv4InIpv6TooFewParts,

    /// <https://url.spec.whatwg.org/#ipv6-invalid-code-point>
    Ipv6InvalidCodepoint,

    /// <https://url.spec.whatwg.org/#ipv6-too-few-pieces>
    Ipv6TooFewPieces,
}

/// <https://url.spec.whatwg.org/#concept-ipv4-parser>
pub(crate) fn ipv4_parse(input: &str) -> Result<net::Ipv4Addr, IPParseError> {
    // Let validationError be false.
    let mut validation_error = false;

    // let parts be the result of strictly splitting input on U+002E (.)
    let mut parts: Vec<&str> = input.split('.').collect();

    // If the last item in parts is the empty string, then:
    if parts.last().copied().is_some_and(str::is_empty) {
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
        // IPv4-too-many-parts validation error, return failure.
        return Err(IPParseError::Ipv4TooManyParts);
    }

    // Let numbers be an empty list.
    let mut numbers = [0; 4];

    // For each part of parts:
    for (index, part) in parts.iter().enumerate() {
        // Let result be the result of parsing part.
        // If result is failure,
        // IPv4-non-numeric-part validation error, return failure.
        let result = ipv4_number_parse(part).map_err(|_| IPParseError::Ipv4NonNumericPart)?;

        // If result[1] is true
        if result.1 {
            // then set validationError to true.
            validation_error = true;
        }

        // Append result[0] to numbers.
        numbers[index] = result.0;
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
        return Err(IPParseError::Ipv4NumberTooLarge);
    }

    // If the last item in numbers is greater than or equal to 256^(5 − numbers’s size),
    if numbers
        .last()
        .is_some_and(|&n| n >= 256_u32.pow(5 - numbers.len() as u32))
    {
        // validation error, return failure.
        return Err(IPParseError::InvalidLastNumber);
    }

    // Let ipv4 be the last item in numbers.
    // Remove the last item from numbers.
    let mut ipv4 = numbers[3];

    // Let counter be 0.
    let mut counter = 0;

    // For each n of numbers:
    #[allow(clippy::explicit_counter_loop)] // Let's follow the spec comments
    for n in numbers.iter().take(3) {
        // Increment ipv4 by n × 256^(3 − counter).
        ipv4 += n * 256_u32.pow(3 - counter);

        // Increment counter by 1.
        counter += 1;
    }

    // Return ipv4.
    Ok(net::Ipv4Addr::from_bits(ipv4))
}

/// <https://url.spec.whatwg.org/#ipv4-number-parser>
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
    // NOTE: rust takes care of that

    // Let output be the mathematical integer value that is represented by input
    // in radix-R notation, using ASCII hex digits for digits with values 0 through 15.
    let output = u32::from_str_radix(input, radix).map_err(|_| ())?;

    // Return (output, validationError).
    Ok((output, validation_error))
}

/// <https://url.spec.whatwg.org/#concept-ipv6-parser>
pub(crate) fn ipv6_parse(input: &str) -> Result<net::Ipv6Addr, IPParseError> {
    // 1. Let address be a new IPv6 address whose IPv6 pieces are all 0.
    let mut address = [0_u16; 8];

    // 2. Let pieceIndex be 0.
    let mut piece_index = 0;

    // 3. Let compress be null.
    let mut compress = None;

    // 4. Let pointer be a pointer for input.
    let mut ptr = 0_usize;

    // 5. If c is U+003A (:), then:
    if input.chars().nth(ptr) == Some(':') {
        // 1. If remaining does not start with U+003A (:),
        if !input[ptr + 1..].starts_with(':') {
            // IPv6-invalid-compression validation error, return failure.
            return Err(IPParseError::Ipv6InvalidCompression);
        }

        // 2. Increase pointer by 2.
        ptr += 2;

        // 3. Increase pieceIndex by 1 and then set compress to pieceIndex.
        piece_index += 1;
        compress = Some(piece_index);
    }

    // 6. While c is not the EOF code point:
    while let Some(c) = input.chars().nth(ptr) {
        // If pieceIndex is 8,
        if piece_index == 8 {
            // IPv6-too-many-pieces validation error, return failure.
            return Err(IPParseError::Ipv6TooManyPieces);
        }

        // 2. If c is U+003A (:), then:
        if c == ':' {
            // 1. If compress is non-null,
            if compress.is_some() {
                // IPv6-multiple-compression validation error, return failure.
                return Err(IPParseError::Ipv6MultipleCompression);
            }

            // 2. Increase pointer and pieceIndex by 1, set compress to pieceIndex, and then continue.
            ptr += 1;
            piece_index += 1;
            compress = Some(piece_index);
            continue;
        }

        // 3. Let value and length be 0.
        let mut value: u16 = 0;
        let mut length = 0;

        // FIXME: This algorithm is *extremely* silly and implements parsing of decimal numbers
        // among other things. It's probably reasonable to not adhere to the spec directly here.

        // 4. While length is less than 4 and c is an ASCII hex digit
        while length < 4
            && let Some(hex_number) = input
                .chars()
                .nth(ptr)
                .as_ref()
                .and_then(|c| c.to_digit(16))
                .and_then(|n| u16::try_from(n).ok())
        {
            // set value to value × 0x10 + c interpreted as hexadecimal number
            value = value * 0x10 + hex_number;

            // and increase pointer and length by 1.
            ptr += 1;
            length += 1;
        }

        // 5. If c is U+002E (.), then:
        if input.chars().nth(ptr) == Some('.') {
            // 1. If length is 0
            if length == 0 {
                // IPv4-in-IPv6-invalid-code-point validation error, return failure.
                return Err(IPParseError::Ipv4InIpv6InvalidCodepoint);
            }

            // 2. Decrease pointer by length.
            ptr -= length;

            // 3. If pieceIndex is greater than 6
            if piece_index > 6 {
                // IPv4-in-IPv6-too-many-pieces validation error, return failure.
                return Err(IPParseError::IPv4InIpv6TooManyParts);
            }

            // 4. Let numbersSeen be 0.
            let mut numbers_seen = 0;

            // 5. While c is not the EOF code point:
            while input.chars().nth(ptr).is_some() {
                // 1. Let ipv4Piece be null.
                let mut ipv4_piece: Option<u16> = None;

                // 2. If numbersSeen is greater than 0, then:
                if numbers_seen > 0 {
                    // 1. If c is a U+002E (.) and numbersSeen is less than 4, then increase pointer by 1.
                    if input.chars().nth(ptr).is_some_and(|c| c == '.') && numbers_seen < 4 {
                        ptr += 1;
                    }
                    // 2. Otherwise, IPv4-in-IPv6-invalid-code-point validation error, return failure.
                    else {
                        return Err(IPParseError::Ipv4InIpv6InvalidCodepoint);
                    }
                }

                // 3. If c is not an ASCII digit, IPv4-in-IPv6-invalid-code-point validation error, return failure.
                if !input
                    .chars()
                    .nth(ptr)
                    .as_ref()
                    .is_some_and(char::is_ascii_digit)
                {
                    return Err(IPParseError::Ipv4InIpv6InvalidCodepoint);
                }

                // 4. While c is an ASCII digit:
                while let Some(number) = input
                    .chars()
                    .nth(ptr)
                    .and_then(|c| c.to_digit(10))
                    .and_then(|n| u16::try_from(n).ok())
                {
                    // 1. Let number be c interpreted as decimal number.

                    // 2. If ipv4Piece is null,then set ipv4Piece to number.
                    //    Otherwise, if ipv4Piece is 0, IPv4-in-IPv6-invalid-code-point validation error, return failure.
                    //    Otherwise, set ipv4Piece to ipv4Piece × 10 + number.
                    match ipv4_piece {
                        None => {
                            ipv4_piece = Some(number);
                        },
                        Some(0) => {
                            return Err(IPParseError::Ipv4InIpv6InvalidCodepoint);
                        },
                        Some(other) => {
                            let new_value = other * 10 + number;

                            // NOTE: This is originally step 3 in the algorithm above, but since this case
                            // can only happen if this branch is taken, we moved it in here.
                            // This also avoids having to use a pointless null check

                            // 3. If ipv4Piece is greater than 255
                            if new_value > 255 {
                                // IPv4-in-IPv6-out-of-range-part validation error, return failure.
                                return Err(IPParseError::Ipv4InIpv6OutOfRangePart);
                            }

                            ipv4_piece = Some(new_value);
                        },
                    };

                    // 4. Increase pointer by 1.
                    ptr += 1;
                }

                // 5. Set address[pieceIndex] to address[pieceIndex] × 0x100 + ipv4Piece.
                address[piece_index] = address[piece_index] * 0x100
                    + ipv4_piece.expect("ipv4Piece cannot be null at this point");

                // 6. Increase numbersSeen by 1.
                numbers_seen += 1;

                // 7. If numbersSeen is 2 or 4, then increase pieceIndex by 1.
                if numbers_seen == 2 || numbers_seen == 4 {
                    piece_index += 1;
                }
            }

            // 6. If numbersSeen is not 4, IPv4-in-IPv6-too-few-parts validation error, return failure.
            if numbers_seen != 4 {
                return Err(IPParseError::Ipv4InIpv6TooFewParts);
            }

            // 7. Break.
            break;
        }
        // 6. Otherwise, if c is U+003A (:):
        else if input.chars().nth(ptr) == Some(':') {
            // Increase pointer by 1.
            ptr += 1;

            // If c is the EOF code point,
            if input.chars().nth(ptr).is_none() {
                // IPv6-invalid-code-point validation error, return failure.
                return Err(IPParseError::Ipv6InvalidCodepoint);
            }
        }
        // 7. Otherwise, if c is not the EOF code point
        else if input.chars().nth(ptr).is_some() {
            // IPv6-invalid-code-point validation error, return failure.
            return Err(IPParseError::Ipv6InvalidCodepoint);
        }

        // 8. Set address[pieceIndex] to value.
        address[piece_index] = value;

        // 9. Increase pieceIndex by 1.
        piece_index += 1
    }

    // 7. If compress is non-null, then:
    if let Some(compress_value) = compress {
        // 1. Let swaps be pieceIndex − compress.
        let mut swaps = piece_index - compress_value;

        // 2. Set pieceIndex to 7.
        piece_index = 7;

        // 3. While pieceIndex is not 0 and swaps is greater than 0
        while piece_index != 0 && swaps > 0 {
            // swap address[pieceIndex] with address[compress + swaps − 1]
            address.swap(piece_index, compress_value + swaps - 1);

            // and then decrease both pieceIndex and swaps by 1.
            piece_index -= 1;
            swaps -= 1;
        }
    }
    // 8. Otherwise, if compress is null and pieceIndex is not 8, IPv6-too-few-pieces validation error, return failure.
    else if piece_index != 8 {
        return Err(IPParseError::Ipv6TooFewPieces);
    }

    // 9. Return address.
    Ok(net::Ipv6Addr::new(
        address[0], address[1], address[2], address[3], address[4], address[5], address[6],
        address[7],
    ))
}

#[cfg(test)]
mod tests {
    use std::net;

    use super::{ipv4_parse, ipv6_parse};

    #[test]
    fn test_ipv4_parse() {
        assert_eq!(ipv4_parse("127.0.0.1"), Ok(net::Ipv4Addr::LOCALHOST));

        // Test parsing with hex numbers
        // This is explicitly forbidden in https://datatracker.ietf.org/doc/html/rfc6943#section-3.1.1
        // but the URL specification allows for it, so we should too.
        let with_hex = net::Ipv4Addr::new(255, 1, 2, 3);
        assert_eq!(ipv4_parse("0xff.1.0x2.3"), Ok(with_hex));
    }

    #[test]
    fn test_ipv6_parse() {
        let ipv6 = net::Ipv6Addr::new(1, 1, 2, 3, 4, 5, 6, 7);
        assert_eq!(ipv6_parse("1.1.2.3.4.5.6.7"), Ok(ipv6));
    }
}
