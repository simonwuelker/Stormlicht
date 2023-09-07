use std::rc::Rc;

use math::Rectangle;

use crate::css::{
    display_list::Painter,
    layout::{CSSPixels, Sides},
    stylecomputer::ComputedStyle,
};

#[derive(Clone, Debug)]
pub struct BoxFragment {
    style: Rc<ComputedStyle>,
    margin: Sides<CSSPixels>,
    border: Sides<CSSPixels>,
    padding: Sides<CSSPixels>,
    children: Vec<Fragment>,
}

#[derive(Clone, Debug)]
pub struct LineBoxFragment {
    rect: Rectangle<CSSPixels>,
    text: String,
}

#[derive(Clone, Debug)]
pub enum Fragment {
    Box(BoxFragment),
    LineBox(LineBoxFragment),
}

impl Fragment {
    pub fn fill_display_list(&self, painter: &mut Painter) {
        match self {
            Self::Box(box_fragment) => box_fragment.fill_display_list(painter),
            Self::LineBox(line_box_fragment) => line_box_fragment.fill_display_list(painter),
        }
    }
}

impl LineBoxFragment {
    #[must_use]
    pub fn new(text: String, rect: Rectangle<CSSPixels>) -> Self {
        Self { text, rect }
    }

    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn fill_display_list(&self, painter: &mut Painter) {
        // FIXME: Paint the line box
    }
}

impl BoxFragment {
    #[must_use]
    pub fn new(
        style: Rc<ComputedStyle>,
        margin: Sides<CSSPixels>,
        border: Sides<CSSPixels>,
        padding: Sides<CSSPixels>,
        children: Vec<Fragment>,
    ) -> Self {
        Self {
            style,
            margin,
            border,
            padding,
            children,
        }
    }

    #[must_use]
    pub fn style(&self) -> Rc<ComputedStyle> {
        self.style.clone()
    }

    #[must_use]
    pub fn children(&self) -> &[Fragment] {
        &self.children
    }

    pub fn fill_display_list(&self, painter: &mut Painter) {
        // FIXME: Paint the box itself

        // Paint all children
        for child in self.children() {
            child.fill_display_list(painter);
        }
    }
}
