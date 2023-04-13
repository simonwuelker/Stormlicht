use std::str::FromStr;

/// <https://url.spec.whatwg.org/#ip-address>
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Ipv4Address(u32);

/// <https://url.spec.whatwg.org/#ip-address>
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Ipv6Address([u16; 8]);

#[derive(Clone, Copy, Debug)]
pub enum IPParseError {
    Empty,
    InvalidDigit,
    /// A value exceeded the maximum of `255`
    Ipv4NumberTooLarge,
    InvalidLastNumber,
    TooManyParts,
    UnexpectedEOF,
    /// Error isn't properly described yet
    Generic,
}

impl FromStr for Ipv4Address {
    type Err = IPParseError;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Ok(Self(ipv4_parse(input)?))
    }
}

impl FromStr for Ipv6Address {
    type Err = IPParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Ok(Self(ipv6_parse(input)?))
    }
}

impl ToString for Ipv4Address {
    fn to_string(&self) -> String {
        // 1. Let output be the empty string.
        let mut octets = [0; 4];

        // 2. Let n be the value of address.
        let mut n = self.0;

        // 3. For each i in the range 1 to 4, inclusive:
        for i in 0..4 {
            // 1. Prepend n % 256, serialized, to output.
            octets[i] = n % 256;
            // 2. If i is not 4, then prepend U+002E (.) to output.
            // NOTE: the actual serialization happens later in the code

            // 3. Set n to floor(n / 256).
            n /= 256;
        }

        // 4 Return output.
        format!("{}.{}.{}.{}", octets[0], octets[1], octets[2], octets[3])
    }
}

impl ToString for Ipv6Address {
    fn to_string(&self) -> String {
        // 1. Let output be the empty string.
        let mut output = String::new();

        // 2. Let compress be an index to the first IPv6 piece in the first longest sequences of address’s IPv6 pieces that are 0.
        let mut longest_sequence_length = 0;
        let mut longest_sequence_start = 0;
        let mut current_sequence_length = 0;
        let mut current_sequence_start = 0;

        for (index, &piece) in self.0.iter().enumerate() {
            if piece == 0 {
                if current_sequence_length == 0 {
                    current_sequence_start = index;
                }

                current_sequence_length += 1;
                if current_sequence_length > longest_sequence_length {
                    longest_sequence_length = current_sequence_length;
                    longest_sequence_start = current_sequence_start;
                }
            } else {
                current_sequence_length = 0;
            }
        }

        // 3. If there is no sequence of address’s IPv6 pieces that are 0 that is longer than 1, then set compress to null.
        let compress = if longest_sequence_length == 1 {
            0
        } else {
            longest_sequence_start
        };

        // 4. Let ignore0 be false.
        let mut ignore0 = false;

        // 5. For each pieceIndex in the range 0 to 7, inclusive:
        for piece_index in 0..8 {
            // 1. If ignore0 is true and address[pieceIndex] is 0, then continue.
            if ignore0 && self.0[piece_index] == 0 {
                continue;
            }

            // 2. Otherwise, if ignore0 is true, set ignore0 to false.
            ignore0 = false;

            // 3. If compress is pieceIndex, then:
            if compress == piece_index {
                // 1. Let separator be "::" if pieceIndex is 0, and U+003A (:) otherwise.
                let seperator = if piece_index == 0 { "::" } else { ":" };

                // 2. Append separator to output.
                output.push_str(seperator);

                // 3. Set ignore0 to true and continue.
                ignore0 = true;
                continue;
            }

            // 4. Append address[pieceIndex], represented as the shortest possible lowercase hexadecimal number, to output.
            output.push_str(&format!("{:x}", self.0[piece_index]));

            // If pieceIndex is not 7, then append U+003A (:) to output.
            output.push(':')
        }

        // 6. Return output.
        output
    }
}

/// <https://url.spec.whatwg.org/#ipv4-number-parser>
fn ipv4_number_parse(mut input: &str) -> Result<(u32, bool), IPParseError> {
    // If input is the empty string,
    if input.is_empty() {
        // then return failure
        return Err(IPParseError::Empty);
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
    let output = u32::from_str_radix(input, radix).map_err(|_| IPParseError::InvalidDigit)?;

    // Return (output, validationError).
    Ok((output, validation_error))
}

/// <https://url.spec.whatwg.org/#concept-ipv4-parser>
fn ipv4_parse(input: &str) -> Result<u32, IPParseError> {
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
        return Err(IPParseError::TooManyParts);
    }

    // Let numbers be an empty list.
    let mut numbers = vec![];

    // For each part of parts:
    for part in parts {
        // Let result be the result of parsing part.
        // If result is failure,
        //      validation error, return failure.
        let result = ipv4_number_parse(part)?;

        // If result[1] is true
        if result.1 {
            // then set validationError to true.
            validation_error = true;
        }

        // Append result[0] to numbers.
        numbers.push(result.0);
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
    if *numbers.last().unwrap() > 256_u32.pow(5 - numbers.len() as u32) {
        // validation error, return failure.
        return Err(IPParseError::InvalidLastNumber);
    }

    // Let ipv4 be the last item in numbers.
    let mut ipv4: u32 = *numbers.last().unwrap();

    // Remove the last item from numbers.
    numbers.pop();

    // Let counter be 0.
    let mut counter = 0;

    // For each n of numbers:
    #[allow(clippy::explicit_counter_loop)] // Let's follow the spec comments
    for n in numbers {
        // Increment ipv4 by n × 256^(3 − counter).
        ipv4 += n * 256_u32.pow(3 - counter);

        // Increment counter by 1.
        counter += 1;
    }

    // Return ipv4.
    Ok(ipv4)
}

/// <https://url.spec.whatwg.org/#concept-ipv6-parser>
fn ipv6_parse(input: &str) -> Result<[u16; 8], IPParseError> {
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
            return Err(IPParseError::Generic);
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
            return Err(IPParseError::Generic);
        }

        // If c is U+003A (:), then:
        if c == ':' {
            // If compress is non-null,
            if compress.is_some() {
                // validation error, return failure.
                return Err(IPParseError::Generic);
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
                return Err(IPParseError::Generic);
            }

            // Decrease pointer by length.
            ptr -= length;

            // If pieceIndex is greater than 6
            if piece_index > 6 {
                // validation error, return failure.
                return Err(IPParseError::TooManyParts);
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
                        return Err(IPParseError::Generic);
                    }
                }

                // If c is not an ASCII digit,
                if !input.chars().nth(ptr).unwrap().is_ascii_digit() {
                    // validation error, return failure.
                    return Err(IPParseError::InvalidDigit);
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
                        return Err(IPParseError::Generic);
                    }
                    // Otherwise
                    else {
                        // set ipv4Piece to ipv4Piece × 10 + number.
                        ipv4_piece = Some(ipv4_piece.unwrap() * 10 + number);
                    }

                    // If ipv4Piece is greater than 255
                    if ipv4_piece.unwrap() > 255 {
                        // validation error, return failure.
                        return Err(IPParseError::Generic);
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
                return Err(IPParseError::Generic);
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
                return Err(IPParseError::UnexpectedEOF);
            }
        }
        // Otherwise, if c is not the EOF code point
        else if input.chars().nth(ptr).is_some() {
            // validation error, return failure.
            return Err(IPParseError::Generic);
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
            address.swap(piece_index, compress_value + swaps - 1);

            // and then decrease both pieceIndex and swaps by 1.
            piece_index -= 1;
            swaps -= 1;
        }
    }
    // Otherwise, if compress is null and pieceIndex is not 8
    else if piece_index != 8 {
        // validation error, return failure.
        return Err(IPParseError::Generic);
    }

    // Return address.
    Ok(address)
}
