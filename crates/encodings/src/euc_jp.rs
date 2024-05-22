//! <https://encoding.spec.whatwg.org/#euc-jp>

use super::{index, Context, DecodeResult, Decoder};
use std::ascii;

/// <https://encoding.spec.whatwg.org/#euc-jp-decoder>
#[derive(Clone, Copy, Debug, Default)]
pub struct EucJpDecoder {
    /// <https://encoding.spec.whatwg.org/#euc-jp-jis0212-flag>
    jis0212: bool,

    /// <https://encoding.spec.whatwg.org/#euc-jp-lead>
    lead: u8,
}

impl Decoder for EucJpDecoder {
    // <https://encoding.spec.whatwg.org/#euc-jp-decoder>
    fn eat_byte(&mut self, context: &mut Context<'_>) -> DecodeResult {
        let byte = context.next();

        // 1. If byte is end-of-queue and EUC-JP lead is not 0x00, set EUC-JP lead to 0x00, and return error.
        // 2. If byte is end-of-queue and EUC-JP lead is 0x00, return finished.
        let Some(byte) = byte else {
            if self.lead != 0 {
                self.lead = 0;
                return DecodeResult::Error;
            } else {
                return DecodeResult::Finished;
            }
        };

        // 3. If EUC-JP lead is 0x8E and byte is in the range 0xA1 to 0xDF, inclusive,
        //    set EUC-JP lead to 0x00 and return a code point whose value is 0xFF61 − 0xA1 + byte.
        if self.lead == 0x8E && matches!(byte, 0xA1..=0xDF) {
            self.lead = 0x00;
            let code_point = 0xFF61 - 0xA1 + byte as u32;
            debug_assert!(char::from_u32(code_point).is_some());
            let c = unsafe { char::from_u32_unchecked(code_point) };
            return DecodeResult::Item(c);
        }

        // 4. If EUC-JP lead is 0x8F and byte is in the range 0xA1 to 0xFE, inclusive,
        //    set EUC-JP jis0212 to true, set EUC-JP lead to byte, and return continue.
        if self.lead == 0x8F && matches!(byte, 0xA1..=0xFE) {
            self.jis0212 = true;
            self.lead = byte;
            return DecodeResult::Continue;
        }

        // 5. If EUC-JP lead is not 0x00, let lead be EUC-JP lead, set EUC-JP lead to 0x00, and then:
        if self.lead != 0 {
            let lead = self.lead;
            self.lead = 0;

            // 1. Let code point be null.
            let mut code_point = '\x00';

            // 2. If lead and byte are both in the range 0xA1 to 0xFE, inclusive,
            //    then set code point to the index code point for
            //    (lead − 0xA1) × 94 + byte − 0xA1 in index jis0208
            //    if EUC-JP jis0212 is false and in index jis0212 otherwise.
            if matches!(lead, 0xA1..=0xFE) && matches!(byte, 0xA1..=0xFE) {
                let pointer = (lead as u16 - 0xA1) * 94 + byte as u16 - 0xA1;

                code_point = if !self.jis0212 {
                    index::jis0208::TABLE[pointer as usize]
                } else {
                    index::jis0212::TABLE[pointer as usize]
                }
            }

            // 3. Set EUC-JP jis0212 to false.
            self.jis0212 = false;

            // 4. If code point is non-null, return a code point whose value is code point.
            if code_point != '\x00' {
                return DecodeResult::Item(code_point);
            }

            // 5. If byte is an ASCII byte, restore byte to ioQueue.
            if byte < 0x80 {
                context.go_back();
            }

            // 6. Return error.
            return DecodeResult::Error;
        }

        // 6. If byte is an ASCII byte, return a code point whose value is byte.
        if let Some(ascii_char) = ascii::Char::from_u8(byte) {
            return DecodeResult::Item(ascii_char.into());
        }

        // 7. If byte is 0x8E, 0x8F, or in the range 0xA1 to 0xFE, inclusive,
        //    set EUC-JP lead to byte and return continue.
        if matches!(byte, 0x8E | 0x8F | 0xA1..=0xFE) {
            self.lead = byte;
            return DecodeResult::Continue;
        }

        // 8. Return error
        DecodeResult::Error
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decoder_valid() {
        // Test case from:
        // https://github.com/lifthrasiir/rust-encoding/blob/master/src/codec/japanese.rs#L230C3-L243C6

        assert_eq!(EucJpDecoder::fully_decode([]).unwrap(), "");
        assert_eq!(EucJpDecoder::fully_decode([0x41]).unwrap(), "A");
        assert_eq!(EucJpDecoder::fully_decode([0x42, 0x43]).unwrap(), "BC");
        assert_eq!(EucJpDecoder::fully_decode([0x5c]).unwrap(), "\\");
        assert_eq!(EucJpDecoder::fully_decode([0x7e]).unwrap(), "~");
        assert_eq!(
            EucJpDecoder::fully_decode([0xa4, 0xcb, 0xa4, 0xdb, 0xa4, 0xf3]).unwrap(),
            "\u{306b}\u{307b}\u{3093}"
        );
        assert_eq!(
            EucJpDecoder::fully_decode([0x8e, 0xc6, 0x8e, 0xce, 0x8e, 0xdd]).unwrap(),
            "\u{ff86}\u{ff8e}\u{ff9d}"
        );
        assert_eq!(
            EucJpDecoder::fully_decode([0xc6, 0xfc, 0xcb, 0xdc]).unwrap(),
            "\u{65e5}\u{672c}"
        );
        assert_eq!(
            EucJpDecoder::fully_decode([0x8f, 0xcb, 0xc6, 0xec, 0xb8]).unwrap(),
            "\u{736c}\u{8c78}"
        );
    }
}
