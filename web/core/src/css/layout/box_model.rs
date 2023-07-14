//! <https://drafts.csswg.org/css2/#box-model>

use math::Rectangle;

#[derive(Clone, Copy, Debug)]
pub struct CSSBox {
    margin_area: Rectangle,
    border_area: Rectangle,
    padding_area: Rectangle,
    content_area: Rectangle,
}

macro_rules! segment_getters {
    (
        #[$fn_top_doc: meta]
        $fn_top: ident, 
        #[$fn_right_doc: meta]
        $fn_right: ident, 
        #[$fn_bottom_doc: meta]
        $fn_bottom: ident, 
        #[$fn_left_doc: meta]
        $fn_left: ident, 
        $outer_rect: ident, 
        $inner_rect: ident
    ) => {
        #[inline]
        #[$fn_top_doc]
        pub fn $fn_top(&self) -> f32 {
            self.$inner_rect.top_left.y - self.$outer_rect.top_left.y
        }

        #[inline]
        #[$fn_right_doc]
        pub fn $fn_right(&self) -> f32 {
            self.$outer_rect.bottom_right.x - self.$inner_rect.bottom_right.x
        }

        #[inline]
        #[$fn_bottom_doc]
        pub fn $fn_bottom(&self) -> f32 {
            self.$outer_rect.bottom_right.y - self.$inner_rect.bottom_right.y
        }

        #[inline]
        #[$fn_left_doc]
        pub fn $fn_left(&self) -> f32 {
            self.$inner_rect.top_left.x - self.$outer_rect.top_left.x
        }
    };
}

impl CSSBox {
    /// Create a new box from the different areas.
    /// 
    /// # Panic
    /// This function panics if the outer rectangles do not
    /// fully contain the inner ones.
    /// (margin contains border contains padding contains content)
    #[inline]
    pub fn new(
        margin_area: Rectangle,
        border_area: Rectangle,
        padding_area: Rectangle,
        content_area: Rectangle,
    ) -> Self {
        assert!(margin_area.contains(border_area));
        assert!(border_area.contains(padding_area));
        assert!(padding_area.contains(content_area));

        Self {
            margin_area,
            border_area,
            padding_area,
            content_area,
        }
    }

    /// <https://drafts.csswg.org/css2/#box-margin-area>
    #[inline]
    pub fn margin_area(&self) -> Rectangle {
        self.margin_area
    }

    /// <https://drafts.csswg.org/css2/#box-border-area>
    #[inline]
    pub fn border_area(&self) -> Rectangle {
        self.border_area
    }

    /// <https://drafts.csswg.org/css2/#box-padding-area>
    #[inline]
    pub fn padding_area(&self) -> Rectangle {
        self.padding_area
    }

    /// <https://drafts.csswg.org/css2/#box-content-area>
    #[inline]
    pub fn content_area(&self) -> Rectangle {
        self.content_area
    }

    segment_getters!(
        /// <https://drafts.csswg.org/css2/#propdef-margin-top>
        margin_top,
        /// <https://drafts.csswg.org/css2/#propdef-margin-right>
        margin_right,
        /// <https://drafts.csswg.org/css2/#propdef-margin-bottom>
        margin_bottom,
        /// <https://drafts.csswg.org/css2/#propdef-margin-left>
        margin_left,
        margin_area,
        border_area
    );

    segment_getters!(
        /// <https://drafts.csswg.org/css2/#propdef-border-top-width>
        border_top_width,
        /// <https://drafts.csswg.org/css2/#propdef-border-right-width>
        border_right_width,
        /// <https://drafts.csswg.org/css2/#propdef-border-bottom-width>
        border_bottom_width,
        /// <https://drafts.csswg.org/css2/#propdef-border-left-width>
        border_left_width,
        border_area,
        padding_area
    );

    segment_getters!(
        /// <https://drafts.csswg.org/css2/#propdef-padding-top>
        padding_top,
        /// <https://drafts.csswg.org/css2/#propdef-padding-right>
        padding_right,
        /// <https://drafts.csswg.org/css2/#propdef-padding-bottom>
        padding_bottom,
        /// <https://drafts.csswg.org/css2/#propdef-padding-left>
        padding_left,
        padding_area,
        content_area
    );
}

#[cfg(test)]
mod tests {
    use super::CSSBox;
    use math::{Rectangle, Vec2D};

    #[test]
    fn dimensions() {
        let margin_area = Rectangle {
            top_left: Vec2D::new(-100., -100.),
            bottom_right: Vec2D::new(100., 100.),
        };
        let border_area = Rectangle {
            top_left: Vec2D::new(-80., -70.),
            bottom_right: Vec2D::new(60., 90.),
        };
        let padding_area = Rectangle {
            top_left: Vec2D::new(-50., -60.),
            bottom_right: Vec2D::new(50., 70.),
        };
        let content_area = Rectangle {
            top_left: Vec2D::new(-30., -50.),
            bottom_right: Vec2D::new(10., 40.),
        };
        let cssbox = CSSBox::new(margin_area, border_area, padding_area, content_area);

        assert_eq!(cssbox.margin_area(), margin_area);
        assert_eq!(cssbox.border_area(), border_area);
        assert_eq!(cssbox.padding_area(), padding_area);
        assert_eq!(cssbox.content_area(), content_area);

        assert_eq!(cssbox.margin_top(), 30.);
        assert_eq!(cssbox.margin_right(), 40.);
        assert_eq!(cssbox.margin_bottom(), 10.);
        assert_eq!(cssbox.margin_left(), 20.);

        assert_eq!(cssbox.border_top_width(), 10.);
        assert_eq!(cssbox.border_right_width(), 10.);
        assert_eq!(cssbox.border_bottom_width(), 20.);
        assert_eq!(cssbox.border_left_width(), 30.);

        assert_eq!(cssbox.padding_top(), 10.);
        assert_eq!(cssbox.padding_right(), 40.);
        assert_eq!(cssbox.padding_bottom(), 30.);
        assert_eq!(cssbox.padding_left(), 20.);
    }
}
