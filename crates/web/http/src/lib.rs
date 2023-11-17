//! A library for handling HTTP/1.1 traffic over an established tcp connection
//!
//! [Specifications](https://developer.mozilla.org/en-US/docs/Web/HTTP/Resources_and_specifications)

#![feature(exclusive_range_pattern)]

mod headers;
pub mod request;
mod response;
mod status_code;

pub use headers::Headers;
pub use request::Request;
pub use response::Response;
pub use status_code::StatusCode;
