mod pattern;
mod str;
mod string;
mod write;

pub use pattern::{DoubleEndedSearcher, Pattern, ReverseSearcher, Searcher};
pub use str::Str;
pub use string::{NotAscii, String};
pub use write::Write;

pub use std::ascii::Char;

pub trait AsciiCharExt {
    /// <https://infra.spec.whatwg.org/#ascii-whitespace>
    fn is_whitespace(&self) -> bool;
    fn is_newline(&self) -> bool;
}

impl AsciiCharExt for Char {
    fn is_whitespace(&self) -> bool {
        matches!(
            self,
            Self::LineTabulation | Self::LineFeed | Self::FormFeed | Self::Space
        )
    }

    fn is_newline(&self) -> bool {
        matches!(self, Char::LineFeed | Char::CarriageReturn)
    }
}
