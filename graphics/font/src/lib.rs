//! Font rasterizing library.
//!
//! In the future, this should also cover [Font Shaping](https://fonts.google.com/knowledge/glossary/shaping)
//! but for now, we are only concerned with rasterization.

#![feature(array_chunks)]

pub mod path;
mod stream;
pub mod ttf;
pub mod ttf_tables;

pub use stream::{Readable, Stream};
pub use ttf::Font;
