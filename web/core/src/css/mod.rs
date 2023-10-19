//! Cascading Style Sheets

mod computed_style;
pub(crate) mod display_list;
mod font_metrics;
pub(crate) mod fragment_tree;
pub(crate) mod layout;
mod line_break;
mod properties;
mod selectors;
mod serialize;
mod stylecomputer;
mod stylesheet;
pub(crate) mod syntax;
mod values;

use computed_style::ComputedStyle;
use font_metrics::FontMetrics;
use line_break::LineBreakIterator;
use properties::{StyleProperty, StylePropertyDeclaration};
use serialize::{Serialize, Serializer};
pub(crate) use stylecomputer::StyleComputer;
pub(crate) use stylesheet::{Origin, StyleRule, Stylesheet};
pub(crate) use syntax::parser::{CSSParse, ParseError, Parser};
