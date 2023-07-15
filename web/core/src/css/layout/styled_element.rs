use crate::{
    css::{
        properties::WidthValue,
        stylecomputer::ComputedStyle,
        values::{Length, LengthPercentage},
    },
    dom::{dom_objects, DOMPtr},
};

use super::ComputedWidthsAndMargins;

#[derive(Clone)]
pub struct StyledElement<'a> {
    element: DOMPtr<dom_objects::Element>,
    style: &'a ComputedStyle,
}

impl<'a> StyledElement<'a> {
    #[inline]
    #[must_use]
    pub fn new(element: DOMPtr<dom_objects::Element>, style: &'a ComputedStyle) -> Self {
        Self { element, style }
    }

    #[inline]
    #[must_use]
    pub fn element(&self) -> DOMPtr<dom_objects::Element> {
        self.element.clone()
    }

    #[inline]
    #[must_use]
    pub fn style(&self) -> &'a ComputedStyle {
        self.style
    }

    /// <https://drafts.csswg.org/css2/#inline-width>
    #[must_use]
    pub fn inline_width(&self) -> ComputedWidthsAndMargins {
        // The width property does not apply.
        let width = None;

        // A computed value of auto for margin-left or margin-right becomes a used value of 0.
        let margin_left = self
            .style
            .margin_left()
            .resolve_auto(LengthPercentage::ZERO);
        let margin_right = self
            .style
            .margin_right()
            .resolve_auto(LengthPercentage::ZERO);

        ComputedWidthsAndMargins {
            width,
            margin_left,
            margin_right,
        }
    }

    /// <https://drafts.csswg.org/css2/#inline-replaced-width>
    #[must_use]
    pub fn inline_replaced_width(&self) -> ComputedWidthsAndMargins {
        // A computed value of auto for margin-left or margin-right becomes a used value of 0.
        let margin_left = self
            .style
            .margin_left()
            .resolve_auto(LengthPercentage::ZERO);
        let margin_right = self
            .style
            .margin_right()
            .resolve_auto(LengthPercentage::ZERO);

        let width = match self.style.width() {
            WidthValue::Auto => {
                if self.style.height().is_auto() {
                    if let Some(intrinsic_size) = self.element.borrow().intrinsic_size() {
                        // If height and width both have computed values of auto and the element
                        // also has an intrinsic width, then that intrinsic width is the used value of width.
                        Some(Length::from_pixels(intrinsic_size.width()).into())
                    } else {
                        log::warn!(
                            "FIXME: Consider intrinsic-height/intrinsic-ratio for computed width",
                        );
                        None
                    }
                } else if let Some(intrinsic_size) = self.element.borrow().intrinsic_size() {
                    // Otherwise, if width has a computed value of auto, and the element has an intrinsic width,
                    // then that intrinsic width is the used value of width.
                    Some(Length::from_pixels(intrinsic_size.width()).into())
                } else {
                    // Otherwise, if width has a computed value of auto, but none of the conditions above are met,
                    // then the used value of width becomes 300px. If 300px is too wide to fit the device,
                    // UAs should use the width of the largest rectangle that has a 2:1 ratio and fits the device instead.
                    Some(Length::from_pixels(300.).into())
                }
            },
            WidthValue::Percentage(p) => Some(LengthPercentage::Percent(p)),
            WidthValue::Lenght(length) => Some(length.into()),
        };

        ComputedWidthsAndMargins {
            width,
            margin_left,
            margin_right,
        }
    }
}
