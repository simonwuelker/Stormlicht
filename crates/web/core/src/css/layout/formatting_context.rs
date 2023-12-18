use std::{fmt, fmt::Write};

use crate::{
    css::{computed_style::ComputedStyle, StyleComputer},
    dom::{dom_objects, DomPtr},
    TreeDebug, TreeFormatter,
};

use super::{
    flow::{self, BlockFormattingContext},
    replaced::ReplacedElement,
};

/// <https://drafts.csswg.org/css-display/#independent-formatting-context>
#[derive(Clone)]
pub(crate) enum IndependentFormattingContext {
    Replaced(ReplacedElement),
    NonReplaced(flow::BlockFormattingContext),
}

impl From<ReplacedElement> for IndependentFormattingContext {
    fn from(value: ReplacedElement) -> Self {
        Self::Replaced(value)
    }
}

impl From<flow::BlockFormattingContext> for IndependentFormattingContext {
    fn from(value: flow::BlockFormattingContext) -> Self {
        Self::NonReplaced(value)
    }
}

impl IndependentFormattingContext {
    #[must_use]
    pub fn create(
        element: DomPtr<dom_objects::Element>,
        style_computer: StyleComputer<'_>,
        element_style: ComputedStyle,
    ) -> Self {
        if let Some(replaced_element) =
            ReplacedElement::try_from(element.clone(), element_style.clone())
        {
            Self::Replaced(replaced_element)
        } else {
            Self::NonReplaced(BlockFormattingContext::build(
                element,
                element_style,
                style_computer,
            ))
        }
    }
}

impl TreeDebug for IndependentFormattingContext {
    fn tree_fmt(&self, formatter: &mut TreeFormatter<'_, '_>) -> fmt::Result {
        match self {
            Self::NonReplaced(bfc) => bfc.tree_fmt(formatter),
            Self::Replaced(_) => {
                formatter.indent()?;
                write!(formatter, "Replaced Content")
            },
        }
    }
}
