mod pattern;
mod str;
mod string;
mod write;

pub use pattern::{DoubleEndedSearcher, Pattern, ReverseSearcher, SearchStep, Searcher};
pub use str::Str;
pub use string::{NotAscii, String};
pub use write::Write;

pub use std::ascii::Char;

/// Extensions for [Char]
pub trait AsciiCharExt {
    /// <https://infra.spec.whatwg.org/#ascii-whitespace>
    fn is_whitespace(&self) -> bool;
    fn is_newline(&self) -> bool;

    /// Checks if the character is in the range `a-z` or `A-Z`, inclusive
    fn is_alphabetic(&self) -> bool;
    fn to_lowercase(&self) -> Self;
}

impl AsciiCharExt for Char {
    fn is_whitespace(&self) -> bool {
        matches!(
            self,
            Self::LineTabulation | Self::LineFeed | Self::FormFeed | Self::Space
        )
    }

    fn is_alphabetic(&self) -> bool {
        (Char::CapitalA..=Char::CapitalZ).contains(self)
            || (Char::SmallA..=Char::SmallZ).contains(self)
    }

    fn is_newline(&self) -> bool {
        matches!(self, Char::LineFeed | Char::CarriageReturn)
    }

    fn to_lowercase(&self) -> Self {
        let byte = *self as u8;
        if byte.is_ascii_uppercase() {
            // SAFETY: These are all still ascii bytes (below 0x80)
            unsafe { Self::from_u8_unchecked(byte + 0x20) }
        } else {
            *self
        }
    }
}
