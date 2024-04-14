#![feature(array_chunks)]

pub mod bmp;
pub mod jpeg;
pub mod png;
mod texture;

pub use texture::{AccessMode, Rgbaf32, Texture};
