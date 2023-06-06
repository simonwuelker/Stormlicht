//! A 2D vector renderer
//!
//! ## Design
//! While performance is obviously nice to have, the focus of this library is on ease of use.
//! This renderer is designed to be used in the browser.
//!
//! ## Related
//! * [Vello](https://github.com/linebender/vello)(GPU-centric, Rust)
//! * [Forma](https://github.com/google/forma)(GPU/CPU, Rust)
//! * [Pathfinder](https://github.com/servo/pathfinder) (Developed for [Servo](https://servo.org/), Rust)
//! * [Skia](https://skia.org/) (Used in Chrome, C++)
//! * [raquote](https://github.com/jrmuizel/raqote)

#![feature(array_windows)]
#![feature(portable_simd)]

mod composition;
mod layer;
mod path;
mod rasterizer;

pub use composition::Composition;
pub use layer::{Layer, Source};
pub use path::{FlattenedPathPoint, Path};
pub use rasterizer::{Mask, Rasterizer};
