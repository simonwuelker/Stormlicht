//! Font rasterizing library.
//!
//! In the future, this should also cover [Font Shaping](https://fonts.google.com/knowledge/glossary/shaping)
//! but for now, we are only concerned with rasterization.
//!
//! ## Usage
//! The API isn't stabilized yet.
//!
//! ```rust,ignore
//! let font = Font::default();
//! let canvas = Canvas::new_uninit(300, 100, PixelFormat::RGB8);
//!
//! // Rasterize the string "abc" at font size 24 in black (all zeros)
//! font.rasterize("abc", &mut canvas, 24, &[0, 0, 0]);
//! ```

pub mod path;
mod rasterizer;
mod stream;
pub mod ttf;
pub mod ttf_tables;

pub use rasterizer::{Point, Rasterizer};
pub use stream::{Readable, Stream};
pub use ttf::Font;
