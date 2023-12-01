//! Font rasterizing library.
//!
//! In the future, this should also cover [Font Shaping](https://fonts.google.com/knowledge/glossary/shaping)
//! but for now, we are only concerned with rasterization.

#![feature(
    array_chunks,
    iter_map_windows,
    result_flattening,
    cfg_match,
    iter_advance_by,
    lazy_cell
)]

mod description;
pub mod hinting;
mod manager;
pub mod path;
pub mod sources;
mod stream;
pub mod ttf;
pub mod ttf_tables;

pub use description::{Family, Properties, Style, Weight};
pub use manager::{FontManager, SystemFont, SYSTEM_FONTS};
pub use stream::{Readable, Stream};
pub use ttf::Font;
