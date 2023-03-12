//! Implements the [CSS Syntax Module Level 3](https://drafts.csswg.org/css-syntax/) draft.

#![feature(exclusive_range_pattern, associated_type_defaults, let_chains)]

pub mod parser;
pub mod selectors;
pub mod stylesheet;
pub mod tokenizer;
pub mod tree;