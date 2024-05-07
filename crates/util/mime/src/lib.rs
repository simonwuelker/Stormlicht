//! <https://mimesniff.spec.whatwg.org/>

// TODO: This spec utilizes a few methods from <https://fetch.spec.whatwg.org>, replace the current implementations
// once fetch is implemented.

#![feature(ascii_char_variants, ascii_char)]

mod mime_type;
mod resource;
mod sniff;
mod sniff_tables;

pub use mime_type::{MIMEParseError, MIMEType};
pub use resource::{Metadata, NoSniff, Resource, ResourceLoadError};
