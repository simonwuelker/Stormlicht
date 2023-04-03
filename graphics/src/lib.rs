//! A 2D vector renderer
//!
//! ## Design
//! While performance is obviously nice to have, the focus of this library is on ease of use.
//! This rendered is designed to be used in the browser.
//!
//! ## Example Code
//! The general rendering pipeline looks like this (incomplete):
//! ```rust
//! # use graphics::{Renderer, Compositor, Color, Vec2D, Path};
//!
//! // The compositor manages the layers that should be rendered
//! let mut compositor = Compositor::default();
//! compositor.get_or_insert_layer(0)
//!     .set_color(Color::rgb(255, 111, 200))
//!     .scale(2., 1.)
//!     .add_path(
//!         Path::new(Vec2D::new(0., 0.))
//!             .line_to(Vec2D::new(1., 0.))
//!             .line_to(Vec2D::new(1., 1.))
//!             .line_to(Vec2D::new(0., 1.))
//!             .close_contour()
//!     );
//!
//! Renderer::render(&mut compositor);
//! ```
//!
//! ## Related
//! * [Vello](https://github.com/linebender/vello)(GPU-centric, Rust)
//! * [Forma](https://github.com/google/forma)(GPU/CPU, Rust)
//! * [Pathfinder](https://github.com/servo/pathfinder) (Developed for [Servo](https://servo.org/), Rust)
//! * [Skia](https://skia.org/) (Used in Chrome, C++)

#![feature(array_windows)]

mod buffer;
mod color;
mod composition;
mod consts;
pub mod math;
mod path;
mod render;

pub use buffer::{Buffer, BufferLayout};
pub use composition::{Compositor, Layer};
pub use path::{FlattenedPathPoint, Path};
pub use render::Renderer;

pub use color::*;
