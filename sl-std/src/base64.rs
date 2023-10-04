use crate::ascii;

const BASE64_CHARS: [ascii::Char; 64] = [
    ascii::Char::CapitalA,
    ascii::Char::CapitalB,
    ascii::Char::CapitalC,
    ascii::Char::CapitalD,
    ascii::Char::CapitalE,
    ascii::Char::CapitalF,
    ascii::Char::CapitalG,
    ascii::Char::CapitalH,
    ascii::Char::CapitalI,
    ascii::Char::CapitalJ,
    ascii::Char::CapitalK,
    ascii::Char::CapitalL,
    ascii::Char::CapitalM,
    ascii::Char::CapitalN,
    ascii::Char::CapitalO,
    ascii::Char::CapitalP,
    ascii::Char::CapitalQ,
    ascii::Char::CapitalR,
    ascii::Char::CapitalS,
    ascii::Char::CapitalT,
    ascii::Char::CapitalU,
    ascii::Char::CapitalV,
    ascii::Char::CapitalW,
    ascii::Char::CapitalX,
    ascii::Char::CapitalY,
    ascii::Char::CapitalZ,
    ascii::Char::SmallA,
    ascii::Char::SmallB,
    ascii::Char::SmallC,
    ascii::Char::SmallD,
    ascii::Char::SmallE,
    ascii::Char::SmallF,
    ascii::Char::SmallG,
    ascii::Char::SmallH,
    ascii::Char::SmallI,
    ascii::Char::SmallJ,
    ascii::Char::SmallK,
    ascii::Char::SmallL,
    ascii::Char::SmallM,
    ascii::Char::SmallN,
    ascii::Char::SmallO,
    ascii::Char::SmallP,
    ascii::Char::SmallQ,
    ascii::Char::SmallR,
    ascii::Char::SmallS,
    ascii::Char::SmallT,
    ascii::Char::SmallU,
    ascii::Char::SmallV,
    ascii::Char::SmallW,
    ascii::Char::SmallX,
    ascii::Char::SmallY,
    ascii::Char::SmallZ,
    ascii::Char::Digit0,
    ascii::Char::Digit1,
    ascii::Char::Digit2,
    ascii::Char::Digit3,
    ascii::Char::Digit4,
    ascii::Char::Digit5,
    ascii::Char::Digit6,
    ascii::Char::Digit7,
    ascii::Char::Digit8,
    ascii::Char::Digit9,
    ascii::Char::PlusSign,
    ascii::Char::Solidus,
];

#[derive(Clone, Copy, Debug)]
pub enum Error {
    IllegalCharacter,
    InvalidLength,
    InvalidPadding,
}

pub fn b64decode(base64: &ascii::Str) -> Result<Vec<u8>, Error> {
    if base64.len() % 4 != 0 {
        return Err(Error::InvalidLength);
    }

    // Let's not worry about this edge case later on
    if base64.is_empty() {
        return Ok(vec![]);
    }

    let padding = if base64[base64.len() - 1] == ascii::Char::EqualsSign {
        if base64[base64.len() - 2] == ascii::Char::EqualsSign {
            2
        } else {
            1
        }
    } else {
        0
    };

    let mut data = Vec::with_capacity((base64.len() / 4) * 3 - padding);
    let mut buffer: u32 = 0;
    let mut iter = 0;

    for &symbol in base64.chars() {
        if symbol == ascii::Char::EqualsSign {
            // End of data
            match iter {
                2 => {
                    data.push((buffer >> 4) as u8);
                },
                3 => {
                    data.push((buffer >> 10) as u8);
                    data.push((buffer >> 2) as u8);
                },
                _ => return Err(Error::InvalidPadding),
            }
            break;
        }

        let index = BASE64_CHARS
            .iter()
            .position(|&c| c == symbol)
            .ok_or(Error::IllegalCharacter)? as u32;
        buffer = (buffer << 6) | index;
        iter += 1;

        if iter == 4 {
            data.extend_from_slice(&buffer.to_be_bytes().as_slice()[1..]);
            iter = 0;
            buffer = 0;
        }
    }

    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::b64decode;

    #[test]
    fn decode() {
        let encoded = "UG9seWZvbiB6d2l0c2NoZXJuZCBhw59lbiBNw6R4Y2hlbnMgVsO2Z2VsIFLDvGJlbiwgSm9naHVydCB1bmQgUXVhcms=".try_into().unwrap();
        let decoded = "Polyfon zwitschernd aßen Mäxchens Vögel Rüben, Joghurt und Quark".as_bytes();

        assert_eq!(b64decode(encoded).unwrap(), decoded);
    }
}
