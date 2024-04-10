//! <https://encoding.spec.whatwg.org>

#![feature(byte_slice_trim_ascii)]

mod encodings {
    include!(concat!(env!("OUT_DIR"), "/encodings.rs"));
}

pub use encodings::Encoding;
