//! Defines properties as defined by the stylesheet author

mod alignment;
mod background_color;
mod background_image;
mod border;
mod cursor;
mod display;
mod float;
mod font_family;
mod font_size;
mod font_style;
mod length;
mod line_height;
mod list_style_type;
mod position;
mod vertical_align;

pub use alignment::{Inset, JustifySelf};
pub use background_color::BackgroundColor;
pub use background_image::BackgroundImage;
pub use border::{Border, LineStyle, LineWidth};
pub use cursor::Cursor;
pub use display::{Display, DisplayBox, DisplayInside, DisplayInsideOutside, DisplayOutside};
pub use float::{Clear, Float, FloatSide};
pub use font_family::{FontFamily, FontName};
pub use font_size::FontSize;
pub use font_style::FontStyle;
pub use length::Length;
pub use line_height::LineHeight;
pub use list_style_type::ListStyleType;
pub use position::Position;
pub use vertical_align::VerticalAlign;

use crate::css::values::{AutoOr, PercentageOr};

/// <https://drafts.csswg.org/css2/#value-def-margin-width>
pub type Margin = AutoOr<PercentageOr<Length>>;

/// <https://drafts.csswg.org/css2/#value-def-padding-width>
pub type Padding = PercentageOr<Length>;
