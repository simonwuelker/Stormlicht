//! Provides various utilities used throughout the [Stormlicht](https://github.com/Wuelle/Stormlicht) codebase.
//!
//! This library can be seen as an extension to the rust standard library.

#![feature(
    array_windows,
    ascii_char,
    ascii_char_variants,
    slice_index_methods,
    const_option,
    round_char_boundary,
    cfg_match,
    bigint_helper_methods,
    array_chunks,
    maybe_uninit_uninit_array,
    assert_matches,
    non_null_convenience
)]

pub mod ascii;
pub mod assert;
pub mod base64;
pub mod big_num;
pub mod bytestream;
pub mod chars;
pub mod datetime;
pub mod fixed;
pub mod iter;
pub mod punycode;
pub mod rand;
pub mod range;
pub mod read;
pub mod ring_buffer;
pub mod safe_casts;
pub mod slice;
