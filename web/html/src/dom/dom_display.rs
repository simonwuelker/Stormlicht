//! Utilities for displaying a dom tree

use std::fmt;

use crate::dom::{dom_objects::Node, DOMPtr};

/// Similar to [Display](std::fmt::Display), except the format is a compact
/// DOM tree, like this:
/// ```text
/// <html>
///     <head>
///     <div>
///         <p>
/// ```
pub trait DOMDisplay {
    fn format(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error>;
}

/// When printing the DOM, we want to keep track of the indent level
/// so the elements appear in a nice tree structure.
/// The default [Debug] trait does not allow for this, which is why we
/// wrap the element to be formatted in an [IndentFormatter].
pub struct IndentFormatter {
    pub indent_level: usize,
    pub inner: DOMPtr<Node>,
}

/// Number of spaces added per nesting level
const INDENT_LEVEL: usize = 2;

impl fmt::Debug for IndentFormatter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let prefix = " ".repeat(self.indent_level);

        // format the node itself
        write!(f, "{prefix}")?;
        crate::dom::codegen::display_domtype(&self.inner, f)?;

        // format all its children
        if !self.inner.borrow().children().is_empty() {
            writeln!(f)?;
        }
        for child in self.inner.borrow().children() {
            let child_fmt = Self {
                indent_level: self.indent_level + INDENT_LEVEL,
                inner: child.clone(),
            };
            writeln!(f, "{prefix}{:?}", child_fmt)?;
        }

        Ok(())
    }
}

// This could be a proc macro but I don't want to raise compile times even more
#[macro_export]
macro_rules! display_tagname {
    ($typename: ident, $tagname: expr) => {
        impl $crate::dom::DOMDisplay for $typename {
            fn format(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
                write!(f, concat!("<", $tagname, ">"))
            }
        }
    };
}

#[macro_export]
macro_rules! display_string {
    ($typename: ident, $string: expr) => {
        impl $crate::dom::DOMDisplay for $typename {
            fn format(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
                write!(f, $string)
            }
        }
    };
}
