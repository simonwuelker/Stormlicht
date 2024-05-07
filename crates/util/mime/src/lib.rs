//! <https://mimesniff.spec.whatwg.org/>

// TODO: This spec utilizes a few methods from <https://fetch.spec.whatwg.org>, replace the current implementations
// once fetch is implemented.

#![feature(ascii_char_variants, ascii_char)]

mod metadata;
mod mime_type;
mod sniff;
mod sniff_tables;

pub use metadata::{Metadata, NoSniff};
pub use mime_type::{MIMEParseError, MIMEType};
