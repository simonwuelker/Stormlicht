//! Cascading Style Sheets

mod properties;
pub mod syntax;

pub mod selectors;
pub mod values;

pub use properties::{StyleProperty, StylePropertyDeclaration};
pub use syntax::parser::{CSSParse, ParseError, Parser};
