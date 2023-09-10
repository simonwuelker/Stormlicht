use std::rc::Rc;

use math::Rectangle;

use crate::css::{
    display_list::Painter,
    layout::{CSSPixels, Sides},
    properties::BackgroundColorValue,
    stylecomputer::ComputedStyle,
};

#[derive(Clone, Debug)]
pub struct BoxFragment {
    style: Rc<ComputedStyle>,
    margin: Sides<CSSPixels>,
    content_area: Rectangle<CSSPixels>,
    children: Vec<Fragment>,
}

#[derive(Clone, Debug)]
pub struct LineBoxFragment {
    _rect: Rectangle<CSSPixels>,
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
        Self { text, _rect: rect }
    }

    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn fill_display_list(&self, _painter: &mut Painter) {
        // FIXME: Paint the line box
    }
}

impl BoxFragment {
    #[must_use]
    pub fn new(
        style: Rc<ComputedStyle>,
        margin: Sides<CSSPixels>,
        content_area: Rectangle<CSSPixels>,
        children: Vec<Fragment>,
    ) -> Self {
        Self {
            style,
            margin,
            content_area,
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

    /// Compute the total space occupied by this fragment, including margins
    pub fn outer_area(&self) -> Rectangle<CSSPixels> {
        self.margin.surround(self.content_area)
    }

    pub fn fill_display_list(&self, painter: &mut Painter) {
        match self.style.background_color() {
            BackgroundColorValue::Inherit => {
                todo!("implement support for background-color: inherit")
            },
            BackgroundColorValue::Transparent => {
                // Skip drawing the background entirely
            },
            BackgroundColorValue::Color(color) => {
                painter.rect(self.content_area, color.into());
            },
        }

        // Paint all children
        for child in self.children() {
            child.fill_display_list(painter);
        }
    }
}
