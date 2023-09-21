use std::rc::Rc;

use math::Rectangle;
use sl_std::range::Range;

use crate::{
    css::{
        display_list::Painter,
        layout::{CSSPixels, Sides},
        properties::BackgroundColorValue,
        stylecomputer::ComputedStyle,
        values::color::Color,
        FontMetrics,
    },
    dom::{self, dom_objects, DOMPtr},
};

#[derive(Clone, Debug)]
pub struct BoxFragment {
    dom_node: Option<DOMPtr<dom_objects::Node>>,
    style: Rc<ComputedStyle>,
    margin: Sides<CSSPixels>,
    content_area: Rectangle<CSSPixels>,
    content_area_including_overflow: Rectangle<CSSPixels>,
    children: Vec<Fragment>,
}

#[derive(Clone, Debug)]
pub struct TextFragment {
    dom_node: DOMPtr<dom_objects::Node>,

    /// The byte offset into the String contained by the text dom node
    /// that produced this fragment
    offset: usize,

    text: String,
    area: Rectangle<CSSPixels>,
    color: Color,
    font_metrics: FontMetrics,
    selected: Option<Range<usize>>,
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
            Self::Text(text_fragment) => {
                let delta_x = point.x - text_fragment.area.top_left.x;
                let font_metrics = FontMetrics::default();
                let text_before = font_metrics.font_face.find_prefix_with_width(
                    text_fragment.text(),
                    font_metrics.size.into(),
                    delta_x.into(),
                );

                let offset = text_before.chars().count();

                Some(dom::BoundaryPoint::new(
                    text_fragment.dom_node.clone(),
                    text_fragment.offset + offset,
                ))
            },
        }
    }
}

impl TextFragment {
    #[must_use]
    pub fn new(
        dom_node: DOMPtr<dom_objects::Node>,
        offset: usize,
        text: String,
        area: Rectangle<CSSPixels>,
        color: Color,
        font_metrics: FontMetrics,
        selected: Option<Range<usize>>,
    ) -> Self {
        Self {
            dom_node,
            offset,
            text,
            area,
            color,
            font_metrics,
            selected,
        }
    }

    #[inline]
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    #[inline]
    pub fn fill_display_list(&self, painter: &mut Painter) {
        let color = if let Some(_selected_range) = self.selected {
            // FIXME: Only paint the relevant range with a different color
            painter.rect(self.area, Color::ORANGE_RED.into());
            math::Color::from(self.color).inverted()
        } else {
            math::Color::from(self.color)
        };

        painter.text(
            self.text.clone(),
            self.area.top_left,
            color,
            self.font_metrics.clone(),
        )
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
        match self.style.background_color() {
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
