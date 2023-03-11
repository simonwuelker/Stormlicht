//! Implements the [CSS Syntax Module Level 3](https://drafts.csswg.org/css-syntax/) draft.

#![feature(exclusive_range_pattern)]

pub mod parser;
mod tokenizer;
pub mod tree;
