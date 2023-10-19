use std::rc::Rc;

use super::{
    layout::Sides,
    values::{AutoOr, BackgroundColor, Color, Display, FontSize, Length, PercentageOr, Position},
};

/// Box data (not inherited)
#[derive(Clone, Debug, Default)]
struct BoxStyleData {
    /// <https://drafts.csswg.org/css-display/#the-display-properties>
    display: Display,

    /// https://drafts.csswg.org/css-position/#position-property>
    position: Position,

    /// <https://drafts.csswg.org/css2/#propdef-width>
    width: AutoOr<PercentageOr<Length>>,

    /// <https://drafts.csswg.org/css2/#propdef-height>
    height: AutoOr<PercentageOr<Length>>,
    // Min Width
    // Min Height
    // Z-Index
}

#[derive(Clone, Copy, Debug, Default)]
struct FontStyleData {
    /// <https://drafts.csswg.org/css2/#font-size-props>
    font_size: FontSize,
}

#[derive(Clone, Debug, Default)]
struct BackgroundData {
    /// <https://drafts.csswg.org/css2/#background-properties>
    background_color: BackgroundColor,
}

/// Miscellaneous, inherited style data
#[derive(Clone, Debug)]
struct InheritedData {
    /// <https://drafts.csswg.org/css2/#colors>
    color: Color,
}

#[derive(Clone, Debug)]
struct SurroundData {
    /// <https://drafts.csswg.org/css-box-3/#propdef-margin>
    margin: Sides<AutoOr<PercentageOr<Length>>>,

    /// <https://drafts.csswg.org/css2/#propdef-padding>
    padding: Sides<PercentageOr<Length>>,
}

#[derive(Clone, Debug, Default)]
pub struct ComputedStyle {
    font_data: Rc<FontStyleData>,
    inherited_data: Rc<InheritedData>,
    surround_data: Rc<SurroundData>,
    box_style_data: Rc<BoxStyleData>,
    background_data: Rc<BackgroundData>,
}

macro_rules! property_access {
    ($name: ident, $set_name: ident, $type: ty, $group_ident: ident.$( $idents: ident ).+) => {
        #[inline]
        #[allow(dead_code)]
        pub fn $name(&self) -> $type {
            self.$group_ident$(.$idents)+
        }

        #[inline]
        #[allow(dead_code)]
        pub fn $set_name(&mut self, value: $type) {
            (*::std::rc::Rc::make_mut(&mut self.$group_ident))$(.$idents)+ = value;
        }
    };
}

macro_rules! property_access_4_sides {
    (
        $sides_name: ident,
        $top_name: ident,
        $right_name: ident,
        $bottom_name: ident,
        $left_name: ident,
        $set_top_name: ident,
        $set_right_name: ident,
        $set_bottom_name: ident,
        $set_left_name: ident,
        $group: ident,
        $type: ty
    ) => {
        property_access!($top_name, $set_top_name, $type, $group.$sides_name.top);
        property_access!(
            $right_name,
            $set_right_name,
            $type,
            $group.$sides_name.right
        );
        property_access!(
            $bottom_name,
            $set_bottom_name,
            $type,
            $group.$sides_name.bottom
        );
        property_access!($left_name, $set_left_name, $type, $group.$sides_name.left);
    };
}
impl ComputedStyle {
    pub fn get_inherited(&self) -> Self {
        Self {
            inherited_data: self.inherited_data.clone(),
            ..Default::default()
        }
    }

    property_access!(
        background_color,
        set_background_color,
        BackgroundColor,
        background_data.background_color
    );
    property_access!(color, set_color, Color, inherited_data.color);
    property_access!(display, set_display, Display, box_style_data.display);
    property_access!(font_size, set_font_size, FontSize, font_data.font_size);
    property_access!(
        height,
        set_height,
        AutoOr<PercentageOr<Length>>,
        box_style_data.height
    );
    property_access!(
        width,
        set_width,
        AutoOr<PercentageOr<Length>>,
        box_style_data.width
    );
    property_access_4_sides!(
        margin,
        margin_top,
        margin_right,
        margin_bottom,
        margin_left,
        set_margin_top,
        set_margin_right,
        set_margin_bottom,
        set_margin_left,
        surround_data,
        AutoOr<PercentageOr<Length>>
    );
    property_access_4_sides!(
        padding,
        padding_top,
        padding_right,
        padding_bottom,
        padding_left,
        set_padding_top,
        set_padding_right,
        set_padding_bottom,
        set_padding_left,
        surround_data,
        PercentageOr<Length>
    );
    property_access!(position, set_position, Position, box_style_data.position);
}

impl Default for InheritedData {
    fn default() -> Self {
        Self {
            // Default "color" is UA dependent (<https://drafts.csswg.org/css2/#colors>)
            color: Color::BLACK,
        }
    }
}

impl Default for SurroundData {
    fn default() -> Self {
        Self {
            margin: Sides::all(AutoOr::NotAuto(PercentageOr::NotPercentage(Length::ZERO))),
            padding: Sides::all(PercentageOr::NotPercentage(Length::ZERO)),
        }
    }
}
