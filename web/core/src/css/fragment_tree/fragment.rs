use std::rc::Rc;

use math::Rectangle;

use crate::{
    css::{
        display_list::Painter,
        layout::{CSSPixels, Sides},
        properties::BackgroundColorValue,
        values::color::Color,
        ComputedStyle, FontMetrics,
    },
    dom::{self, dom_objects, DOMPtr},
};

#[derive(Clone, Debug)]
pub struct BoxFragment {
    /// The [DOM Node](dom) that produced this fragment
    dom_node: Option<DOMPtr<dom_objects::Node>>,

    style: Rc<ComputedStyle>,
    margin: Sides<CSSPixels>,

    /// Content area including padding
    content_area: Rectangle<CSSPixels>,

    content_area_including_overflow: Rectangle<CSSPixels>,
    children: Vec<Fragment>,
}

#[derive(Clone, Debug)]
pub struct TextFragment {
    text: String,
    area: Rectangle<CSSPixels>,
    color: Color,
    font_metrics: FontMetrics,
}

#[derive(Clone, Debug)]
pub enum Fragment {
    Box(BoxFragment),
    Text(TextFragment),
}

impl Fragment {
    pub fn fill_display_list(&self, painter: &mut Painter) {
        match self {
            Self::Box(box_fragment) => box_fragment.fill_display_list(painter),
            Self::Text(text_fragment) => text_fragment.fill_display_list(painter),
        }
    }

    pub fn content_area_including_overflow(&self) -> Rectangle<CSSPixels> {
        match self {
            Self::Box(box_fragment) => box_fragment.content_area_including_overflow,
            Self::Text(text_fragment) => text_fragment.area,
        }
    }

    pub fn hit_test(&self, point: math::Vec2D<CSSPixels>) -> Option<dom::BoundaryPoint> {
        match self {
            Self::Box(box_fragment) => {
                let first_hit_child = box_fragment.children().iter().find(|child| {
                    child
                        .content_area_including_overflow()
                        .contains_point(point)
                });

                if let Some(hit_child) = first_hit_child {
                    hit_child.hit_test(point)
                } else {
                    // None of our children were hit (or we have no children)
                    // In this case, this fragment is the target of the hit-test
                    Some(dom::BoundaryPoint::new(box_fragment.dom_node.clone()?, 0))
                }
            },
            Self::Text(_text_fragment) => {
                // FIXME: Text element hit-testing
                None
            },
        }
    }
}

impl TextFragment {
    #[must_use]
    pub fn new(
        text: String,
        area: Rectangle<CSSPixels>,
        color: Color,
        font_metrics: FontMetrics,
    ) -> Self {
        Self {
            text,
            area,
            color,
            font_metrics,
        }
    }

    #[inline]
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    #[inline]
    pub fn fill_display_list(&self, painter: &mut Painter) {
        let color = math::Color::from(self.color);

        painter.text(
            self.text.clone(),
            self.area.top_left,
            color,
            self.font_metrics.clone(),
        );
    }
}

impl BoxFragment {
    #[must_use]
    pub fn new(
        dom_node: Option<DOMPtr<dom_objects::Node>>,
        style: Rc<ComputedStyle>,
        margin: Sides<CSSPixels>,
        content_area: Rectangle<CSSPixels>,
        content_area_including_overflow: Rectangle<CSSPixels>,
        children: Vec<Fragment>,
    ) -> Self {
        Self {
            dom_node,
            style,
            margin,
            content_area,
            content_area_including_overflow,
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

    #[inline]
    #[must_use]
    pub fn inner_area(&self) -> Rectangle<CSSPixels> {
        self.content_area
    }

    /// Compute the total space occupied by this fragment, including margins
    #[inline]
    #[must_use]
    pub fn outer_area(&self) -> Rectangle<CSSPixels> {
        self.margin.surround(self.content_area)
    }

    #[inline]
    #[must_use]
    pub fn content_area_including_overflow(&self) -> Rectangle<CSSPixels> {
        self.content_area_including_overflow
    }

    pub fn fill_display_list(&self, painter: &mut Painter) {
        match self.style().background_color() {
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
