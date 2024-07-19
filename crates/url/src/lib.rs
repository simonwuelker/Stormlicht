//! Contains parsers and utilities related to **U**niform **R**esource **L**ocators ([URL]s).
//!
//! You can find the relevant specification [here](https://url.spec.whatwg.org/).
//!
//! The preferred way to obtain a [URL] is to parse it like this:
//! ```
//! # use crate::url::URL;
//! let url: URL = "https://google.com".parse().unwrap();
//!
//! assert_eq!(url.scheme(), "https");
//! ```

#![feature(
    let_chains,
    option_get_or_insert_default,
    ascii_char,
    ascii_char_variants,
    string_remove_matches,
    const_option
)]

mod host;
mod ip;
mod parser;
mod percent_encode;
mod set;
mod url;
mod util;

pub use crate::ip::IPParseError;
pub use crate::parser::*;
pub use crate::url::*;
pub use host::Host;
pub use percent_encode::{percent_decode, percent_encode};
use set::AsciiSet;
