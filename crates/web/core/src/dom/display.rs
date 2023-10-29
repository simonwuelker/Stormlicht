//! Utilities for displaying a dom tree

use std::{fmt, fmt::Write};

use crate::{
    dom::{codegen, dom_objects, DOMPtr, DOMTyped},
    TreeDebug,
};

/// Maximum number of text characters to display before cutting them of.
/// This prevents `<script>`/`<style>` spam.
const MAX_TEXT_LEN: usize = 16;

/// Similar to [Display](std::fmt::Display), except the format is a compact
/// DOM tree, like this:
/// ```text
/// <html>
///     <head>
///     <div>
///         <p>
/// ```
pub trait DOMDisplay {
    fn format<W: fmt::Write>(&self, writer: &mut W) -> fmt::Result;
    fn format_text<W: fmt::Write>(&self, writer: &mut W, text: &str) -> fmt::Result {
        if text.len() < MAX_TEXT_LEN {
            write!(writer, "{}", text)
        } else {
            write!(writer, "{} [...]", &text[..MAX_TEXT_LEN])
        }
    }
}

// This could be a proc macro but I don't want to raise compile times even more
#[macro_export]
macro_rules! display_tagname {
    ($typename: ident, $tagname: expr) => {
        impl $crate::dom::DOMDisplay for $typename {
            fn format<W: ::std::fmt::Write>(&self, writer: &mut W) -> ::std::fmt::Result {
                write!(writer, concat!("<", $tagname, ">"))
            }
        }
    };
}

#[macro_export]
macro_rules! display_string {
    ($typename: ident, $string: expr) => {
        impl $crate::dom::DOMDisplay for $typename {
            fn format<W: ::std::fmt::Write>(&self, writer: &mut W) -> ::std::fmt::Result {
                write!(writer, $string)
            }
        }
    };
}

impl<T> TreeDebug for DOMPtr<T>
where
    T: DOMDisplay + DOMTyped,
{
    fn tree_fmt(&self, formatter: &mut crate::TreeFormatter<'_, '_>) -> std::fmt::Result {
        if let Some(node) = self.try_into_type::<dom_objects::Node>() {
            formatter.indent()?;
            codegen::display_domtype(&node, formatter)?;
            writeln!(formatter)?;

            let borrowed_node = node.borrow();
            if !borrowed_node.children().is_empty() {
                formatter.increase_indent();
                for child in borrowed_node.children() {
                    formatter.indent()?;
                    child.tree_fmt(formatter)?;
                }
                formatter.decrease_indent();
            }
        }
        Ok(())
    }
}
