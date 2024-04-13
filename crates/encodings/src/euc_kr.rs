use super::{index, Context, DecodeResult, Decoder};
use std::ascii;

#[derive(Clone, Copy, Debug, Default)]
pub struct EucKrDecoder {
    /// <https://encoding.spec.whatwg.org/#euc-kr-lead>
    lead: u8,
}

impl Decoder for EucKrDecoder {
    // <https://encoding.spec.whatwg.org/#euc-kr-decoder>
    fn eat_byte(&mut self, context: &mut Context<'_>) -> DecodeResult {
        let byte = context.next();

        // 1. If byte is end-of-queue and EUC-KR lead is not 0x00, set EUC-KR lead to 0x00 and return error.
        // 2. If byte is end-of-queue and EUC-KR lead is 0x00, return finished.
        let Some(byte) = byte else {
            if self.lead != 0 {
                self.lead = 0;
                return DecodeResult::Error;
            } else {
                return DecodeResult::Finished;
            }
        };

        // 3. If EUC-KR lead is not 0x00, let lead be EUC-KR lead, let pointer be null,
        // set EUC-KR lead to 0x00, and then:
        if self.lead != 0 {
            let lead = self.lead;
            self.lead = 0;

            // 1. If byte is in the range 0x41 to 0xFE, inclusive, set pointer to (lead − 0x81) × 190 + (byte − 0x41).
            let pointer = if (0x41..=0xFE).contains(&byte) {
                (lead as u16 - 0x81) * 190 + (byte as u16 - 0x41)
            } else {
                0
            };

            // 2. Let code point be null if pointer is null,
            // otherwise the index code point for pointer in index EUC-KR.
            let code_point = if pointer == 0 {
                '\x00'
            } else {
                index::euc_kr::TABLE[pointer as usize]
            };

            // 3. If code point is non-null, return a code point whose value is code point.
            if code_point != '\x00' {
                return DecodeResult::Item(code_point);
            }

            // 4. If byte is an ASCII byte, restore byte to ioQueue.
            if byte >= 0x80 {
                context.go_back();
            }

            // 5. Return error.
            return DecodeResult::Error;
        }

        // 4. If byte is an ASCII byte, return a code point whose value is byte.
        if let Some(ascii_char) = ascii::Char::from_u8(byte) {
            return DecodeResult::Item(ascii_char.into());
        }

        // 5. If byte is in the range 0x81 to 0xFE, inclusive, set EUC-KR lead to byte and return continue.
        if (0x81..=0xFE).contains(&byte) {
            self.lead = byte;
            return DecodeResult::Continue;
        }

        // 6. Return error
        return DecodeResult::Error;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decoder_valid() {
        // Test case from:
        // https://github.com/lifthrasiir/rust-encoding/blob/master/src/codec/korean.rs#L165C1-L176C6

        assert_eq!(EucKrDecoder::fully_decode([]).unwrap(), "");
        assert_eq!(EucKrDecoder::fully_decode([0x41]).unwrap(), "A");
        assert_eq!(EucKrDecoder::fully_decode([0x42, 0x43]).unwrap(), "BC");
        assert_eq!(
            EucKrDecoder::fully_decode([0xb0, 0xa1]).unwrap(),
            "\u{ac00}"
        );
        assert_eq!(
            EucKrDecoder::fully_decode([0xb3, 0xaa, 0xb4, 0xd9]).unwrap(),
            "\u{b098}\u{b2e4}"
        );
        assert_eq!(
            EucKrDecoder::fully_decode([0x94, 0xee, 0xa4, 0xbb, 0xc6, 0x52, 0xc1, 0x64]).unwrap(),
            "\u{bdc1}\u{314b}\u{d7a3}\u{d58f}"
        );
    }
}
