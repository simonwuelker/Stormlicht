use std::rc::Rc;

use crate::{
    css::stylecomputer::ComputedStyle,
    dom::{dom_objects, DOMPtr},
};

/// <https://drafts.csswg.org/css2/#inline-level-boxes>
#[derive(Clone, Debug)]
pub enum InlineLevelBox {
    InlineBox(InlineBox),
    TextRun(String),
}

/// <https://drafts.csswg.org/css2/#inline-box>
#[derive(Clone, Debug)]
pub struct InlineBox;

/// <https://drafts.csswg.org/css2/#inline-formatting>
#[derive(Clone, Debug)]
pub struct InlineFormattingContext {
    contents: Vec<InlineLevelBox>,
}

impl From<Vec<InlineLevelBox>> for InlineFormattingContext {
    fn from(contents: Vec<InlineLevelBox>) -> Self {
        Self { contents }
    }
}

impl InlineLevelBox {
    pub fn from_element(element: DOMPtr<dom_objects::Element>, style: Rc<ComputedStyle>) -> Self {
        debug_assert!(style.display().is_inline());
        _ = element;
        todo!()
    }
}
