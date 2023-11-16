use math::Rectangle;

use crate::{
    css::{
        display_list::Painter,
        layout::{Pixels, Sides},
        values::{BackgroundColor, Color},
        ComputedStyle, FontMetrics,
    },
    dom::{self, dom_objects, DOMPtr},
};

#[derive(Clone, Debug)]
pub struct BoxFragment {
    /// The [DOM Node](dom) that produced this fragment
    dom_node: Option<DOMPtr<dom_objects::Node>>,

    style: ComputedStyle,
    margin_area: Rectangle<Pixels>,
    borders: Sides<Pixels>,
    padding_area: Rectangle<Pixels>,
    content_area_including_overflow: Rectangle<Pixels>,
    children: Vec<Fragment>,
}

#[derive(Clone, Debug)]
pub struct TextFragment {
    text: String,
    area: Rectangle<Pixels>,
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

    pub const fn content_area_including_overflow(&self) -> Rectangle<Pixels> {
        match self {
            Self::Box(box_fragment) => box_fragment.content_area_including_overflow,
            Self::Text(text_fragment) => text_fragment.area,
        }
    }

    pub fn hit_test(&self, point: math::Vec2D<Pixels>) -> Option<dom::BoundaryPoint> {
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
        area: Rectangle<Pixels>,
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
            self.text().to_owned(),
            self.area.top_left(),
            color,
            self.font_metrics.clone(),
        );
    }
}

impl BoxFragment {
    #[must_use]
    pub fn new(
        dom_node: Option<DOMPtr<dom_objects::Node>>,
        style: ComputedStyle,
        margin_area: Rectangle<Pixels>,
        borders: Sides<Pixels>,
        padding_area: Rectangle<Pixels>,
        content_area_including_overflow: Rectangle<Pixels>,
        children: Vec<Fragment>,
    ) -> Self {
        Self {
            dom_node,
            style,
            margin_area,
            borders,
            padding_area,
            content_area_including_overflow,
            children,
        }
    }

    #[must_use]
    pub fn style(&self) -> &ComputedStyle {
        &self.style
    }

    #[must_use]
    pub fn children(&self) -> &[Fragment] {
        &self.children
    }

    /// Compute the total space occupied by this fragment, including margins
    #[inline]
    #[must_use]
    pub fn margin_area(&self) -> Rectangle<Pixels> {
        self.margin_area
    }

    pub fn border_area(&self) -> Rectangle<Pixels> {
        self.borders.surround(self.padding_area)
    }

    #[inline]
    #[must_use]
    pub fn content_area_including_overflow(&self) -> Rectangle<Pixels> {
        self.content_area_including_overflow
    }

    pub fn fill_display_list(&self, painter: &mut Painter) {
        match *self.style().background_color() {
            BackgroundColor::Transparent => {
                // Skip drawing the background entirely
            },
            BackgroundColor::Color(color) => {
                painter.rect(self.padding_area, color.into());
            },
        }

        // Draw borders
        // FIXME: different border styles (other than "solid")
        let border_area = self.border_area();

        // Top border
        if !self.style().border_top_style().is_none() {
            let bottom_right = border_area.top_right()
                + math::Vec2D {
                    x: Pixels::ZERO,
                    y: self.borders.top,
                };
            let area = Rectangle::from_corners(border_area.top_left(), bottom_right);
            let color = *self.style().border_top_color();
            painter.rect(area, color.into());
        }

        // Right border
        if !self.style().border_right_style().is_none() {
            let top_left = border_area.top_right()
                - math::Vec2D {
                    x: self.borders.right,
                    y: Pixels::ZERO,
                };
            let area = Rectangle::from_corners(top_left, border_area.bottom_right());
            let color = *self.style().border_right_color();
            painter.rect(area, color.into());
        }

        // Bottom border
        if !self.style().border_bottom_style().is_none() {
            let top_left = border_area.bottom_left()
                - math::Vec2D {
                    x: Pixels::ZERO,
                    y: self.borders.bottom,
                };
            let area = Rectangle::from_corners(top_left, border_area.bottom_right());
            let color = *self.style().border_bottom_color();
            painter.rect(area, color.into());
        }

        // Left border
        if !self.style().border_left_style().is_none() {
            let bottom_right = border_area.bottom_left()
                + math::Vec2D {
                    x: self.borders.left,
                    y: Pixels::ZERO,
                };
            let area = Rectangle::from_corners(border_area.top_left(), bottom_right);
            let color = *self.style().border_left_color();
            painter.rect(area, color.into());
        }

        // Paint all children
        for child in self.children() {
            child.fill_display_list(painter);
        }
    }
}
