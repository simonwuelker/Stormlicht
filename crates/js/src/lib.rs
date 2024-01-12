#![feature(iter_advance_by)]

pub mod error;
pub mod grammar;
pub mod identifiers;
pub mod literals;
pub mod tokenizer;

pub use grammar::Script;
pub use tokenizer::Tokenizer;
