//! Implements the [CSS Syntax Module Level 3](https://drafts.csswg.org/css-syntax/) draft.

pub mod parser;
pub mod rule_parser;
pub mod selectors;
pub mod tokenizer;
pub mod tree;

pub use parser::Parser;
