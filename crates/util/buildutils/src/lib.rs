//! Provides utilities used by various `build.rs` files around stormlicht

#![feature(lazy_cell)]

mod python;

pub use python::PYTHON;
