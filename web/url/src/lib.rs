//! Contains parsers and utilities related to **U**niform **R**esource **L**ocators ([URL]s).
//!
//! You can find the relevant specification [here](https://url.spec.whatwg.org/).
//!
//! The preferred way to obtain a [URL] is to parse it like this:
//! ```
//! # use crate::url::URL;
//! let url: URL = "https://google.com".try_into().unwrap();
//!
//! assert_eq!(url.scheme(), "https");
//! ```

#![feature(
    let_chains,
    option_get_or_insert_default,
    ascii_char,
    ascii_char_variants,
    ip_bits
)]

mod host;
mod ip;
mod parser;
pub mod percent_encode;
mod url;
mod util;
mod validation_error;

pub use crate::ip::IPParseError;
pub use crate::parser::*;
pub use crate::url::*;
pub use host::Host;
pub use validation_error::{IgnoreValidationErrors, ValidationError, ValidationErrorHandler};
