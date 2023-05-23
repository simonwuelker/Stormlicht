//! Implements the [CSS Syntax Module Level 3](https://drafts.csswg.org/css-syntax/) draft.

pub mod parser;
mod tokenizer;

pub use parser::WhitespaceAllowed;
pub use tokenizer::{HashFlag, Token};
