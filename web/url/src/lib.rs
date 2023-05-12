//! Contains parsers and utilities related to **U**niform **R**esource **L**ocators ([URL]s).
//!
//! You can find the relevant specification [here](https://url.spec.whatwg.org/).
//!
//! The preferred way to obtain a [URL] is to parse it like this:
//! ```
//! # use crate::url::URL;
//! let url: URL = "https://google.com".try_into().unwrap();
//!
//! assert_eq!(url.scheme, "https");
//! ```

mod host;
mod ip;
mod url;
pub mod urlencode;
mod urlparser;
mod util;

pub use crate::ip::{IPParseError, Ipv4Address, Ipv6Address};
pub use crate::url::*;
pub use crate::urlparser::*;
pub use host::Host;
