//! Font rasterizing library.
//!
//! In the future, this should also cover [Font Shaping](https://fonts.google.com/knowledge/glossary/shaping)
//! but for now, we are only concerned with rasterization.

#![feature(array_chunks, iter_map_windows, result_flattening, cfg_match)]

pub mod path;
pub mod sources;
mod stream;
pub mod ttf;
pub mod ttf_tables;

pub use sources::SystemStore;

pub use stream::{Readable, Stream};
pub use ttf::Font;
