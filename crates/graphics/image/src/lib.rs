#![feature(array_chunks, generic_nonzero, non_zero_count_ones)]

pub mod bmp;
pub mod jpeg;
pub mod png;
mod texture;

pub use texture::{AccessMode, Rgbaf32, Texture};
