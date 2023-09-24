//! Provides various utilities used throughout the [Stormlicht](https://github.com/Wuelle/Stormlicht) codebase.
//!
//! This library can be seen as an extension to the rust standard library.

#![feature(
    array_windows,
    ascii_char,
    ascii_char_variants,
    slice_index_methods,
    const_option,
    round_char_boundary
)]

pub mod ascii;
pub mod big_num;
pub mod chars;
pub mod iter;
pub mod punycode;
pub mod rand;
pub mod range;
pub mod slice;
pub mod time;
