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

/// Allows for easy definition of ascii-strings
///
/// Since rust does not allow us to define our own string literals, the use of a macro
/// is necessary. If the provided literal contains non-ascii data then the invocation will
/// panic at compile-time.
///
/// # Example
/// ```
/// # #![feature(ascii_char)]
/// # use sl_std::ascii;
///
/// let foo: &ascii::Str = ascii!("foo");
/// assert_eq!(foo.as_str(), "foo");
///
/// ```
#[macro_export]
macro_rules! ascii {
    ($s: expr) => {
        const {
            match $s.as_bytes().as_ascii() {
                Some(ascii) => $crate::ascii::Str::from_ascii_chars(ascii),
                None => panic!("string literal is not ascii"),
            }
        }
    };
}
