use std::rc::Rc;

use math::Rectangle;

use crate::css::{
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

impl LineBoxFragment {
    #[must_use]
    pub fn new(text: String, rect: Rectangle<CSSPixels>) -> Self {
        Self { text, rect }
    }

    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
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
}
