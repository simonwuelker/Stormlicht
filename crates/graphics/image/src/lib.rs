#![feature(
    array_chunks,
    generic_nonzero,
    non_zero_count_ones,
    const_fn_floating_point_arithmetic
)]

pub mod bmp;
pub mod jpeg;
pub mod png;
mod texture;

pub use texture::{AccessMode, Rgbaf32, Texture};
