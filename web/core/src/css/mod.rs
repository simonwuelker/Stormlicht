//! Cascading Style Sheets

mod properties;
pub mod syntax;

pub mod rule_parser;
pub mod selectors;
pub mod values;

pub use properties::StyleProperty;
pub use syntax::parser::{CSSParse, ParseError, Parser};
