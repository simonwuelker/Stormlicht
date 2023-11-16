use crate::{
    css::{
        layout::Pixels,
        syntax::Token,
        values::{Length, PercentageOr},
        CSSParse, ParseError, Parser,
    },
    static_interned,
};

use super::length;

/// The default font size.
pub const FONT_MEDIUM_PX: Pixels = Pixels(16.0);

/// Ratio applied for font-size: larger/smaller
///
/// Take from https://github.com/servo/servo/blob/fd31da9102497cfaf5265bbab17df4424a8a1078/components/style/values/specified/font.rs#L779
const LARGER_FONT_SIZE_RATIO: f32 = 1.2;

/// <https://drafts.csswg.org/css2/#value-def-absolute-size>
#[derive(Clone, Copy, Debug)]
pub enum AbsoluteSize {
    /// <https://drafts.csswg.org/css2/#valdef-font-size-xx-small>
    XXSmall,

    /// <https://drafts.csswg.org/css2/#valdef-font-size-x-small>
    XSmall,

    /// <https://drafts.csswg.org/css2/#valdef-font-size-small>
    Small,

    /// <https://drafts.csswg.org/css2/#valdef-font-size-medium>
    Medium,

    /// <https://drafts.csswg.org/css2/#valdef-font-size-large>
    Large,

    /// <https://drafts.csswg.org/css2/#valdef-font-size-x-large>
    XLarge,

    /// <https://drafts.csswg.org/css2/#valdef-font-size-xx-large>
    XXLarge,
}

/// <https://drafts.csswg.org/css2/#value-def-relative-size>
#[derive(Clone, Copy, Debug)]
pub enum RelativeSize {
    Smaller,
    Larger,
}

/// <https://drafts.csswg.org/css2/#font-size-props>
#[derive(Clone, Copy, Debug)]
pub enum FontSize {
    Absolute(AbsoluteSize),
    Relative(RelativeSize),
    LengthPercentage(PercentageOr<Length>),
}

/// Contains all values that the absolute value of a [FontSize] may depend on
#[derive(Clone, Copy, Debug)]
pub struct ResolutionContext {
    pub inherited_font_size: Pixels,
    pub length_context: length::ResolutionContext,
}

impl Default for FontSize {
    fn default() -> Self {
        Self::Absolute(AbsoluteSize::Medium)
    }
}

impl AbsoluteSize {
    #[inline]
    const fn html_size(&self) -> u8 {
        *self as u8
    }

    fn to_pixels(self) -> Pixels {
        /// Mapping from html size to scale factor, copied from
        /// https://github.com/servo/servo/blob/fd31da9102497cfaf5265bbab17df4424a8a1078/components/style/values/specified/font.rs#L869
        const FONT_SIZE_FACTORS: [f32; 8] = [0.6, 0.75, 0.89, 1.00, 1.20, 1.50, 2.00, 3.00];

        FONT_MEDIUM_PX * FONT_SIZE_FACTORS[self.html_size() as usize]
    }
}

impl RelativeSize {
    fn to_pixels(self, inherited_font_size: Pixels) -> Pixels {
        match self {
            Self::Smaller => inherited_font_size / LARGER_FONT_SIZE_RATIO,
            Self::Larger => inherited_font_size * LARGER_FONT_SIZE_RATIO,
        }
    }
}

impl FontSize {
    pub fn to_pixels(self, ctx: ResolutionContext) -> Pixels {
        match self {
            Self::Absolute(absolute_size) => absolute_size.to_pixels(),
            Self::Relative(relative_size) => relative_size.to_pixels(ctx.inherited_font_size),
            Self::LengthPercentage(percentage_or_length) => {
                let length =
                    percentage_or_length.resolve_against(Length::pixels(ctx.inherited_font_size));
                length.absolutize(ctx.length_context)
            },
        }
    }
}

impl<'a> CSSParse<'a> for FontSize {
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let position = match parser.peek_token() {
            Some(Token::Ident(static_interned!("xx-small"))) => {
                Self::Absolute(AbsoluteSize::XXSmall)
            },
            Some(Token::Ident(static_interned!("x-small"))) => Self::Absolute(AbsoluteSize::XSmall),
            Some(Token::Ident(static_interned!("small"))) => Self::Absolute(AbsoluteSize::Small),
            Some(Token::Ident(static_interned!("medium"))) => Self::Absolute(AbsoluteSize::Medium),
            Some(Token::Ident(static_interned!("large"))) => Self::Absolute(AbsoluteSize::Large),
            Some(Token::Ident(static_interned!("x-large"))) => Self::Absolute(AbsoluteSize::XLarge),
            Some(Token::Ident(static_interned!("xx-large"))) => {
                Self::Absolute(AbsoluteSize::XXLarge)
            },
            Some(Token::Ident(static_interned!("smaller"))) => {
                Self::Relative(RelativeSize::Smaller)
            },
            Some(Token::Ident(static_interned!("larger"))) => Self::Relative(RelativeSize::Larger),
            _ => return Ok(Self::LengthPercentage(CSSParse::parse(parser)?)),
        };
        parser.next_token();

        Ok(position)
    }
}
