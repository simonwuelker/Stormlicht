#![feature(convert_float_to_int)]

mod format;
pub mod jpeg;
pub mod png;
mod texture;

pub use format::{ColorChannel, ColorFormat, DynamicTexture, Rgb, Rgba};
pub use texture::{AccessMode, Texture};
