//! Implements various encoding schemes

#![feature(cursor_remaining)]

pub mod brotli;
pub mod deflate;
pub mod zlib;

pub mod bit_reader;
pub mod gzip;
pub mod huffman;
pub mod ringbuffer;
