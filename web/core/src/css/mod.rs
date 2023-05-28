//! Cascading Style Sheets

mod properties;
pub mod selectors;
mod stylesheet;
pub mod syntax;
pub mod values;

pub use properties::{StyleProperty, StylePropertyDeclaration};
pub use stylesheet::{Origin, StyleRule, Stylesheet};
pub use syntax::parser::{CSSParse, ParseError, Parser};
