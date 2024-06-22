use crate::{
    css::{
        style::{computed, StyleContext, ToComputedStyle},
        syntax::Token,
        values::Angle,
        CSSParse, ParseError, Parser,
    },
    static_interned,
};

const DEFAULT_OBLIQUE_ANGLE: Angle = Angle::from_degrees(14.);

/// <https://drafts.csswg.org/css-fonts/#font-style-prop>
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique(Angle),
}

impl<'a> CSSParse<'a> for FontStyle {
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let font_style = match parser.expect_identifier()? {
            static_interned!("normal") => Self::Normal,
            static_interned!("italic") => Self::Italic,
            static_interned!("oblique") => {
                let angle = if let Some(Token::Dimension(value, dimension)) =
                    parser.peek_token_ignoring_whitespace(0)
                {
                    if let Ok(angle) = Angle::from_dimension(*value, *dimension) {
                        _ = parser.next_token_ignoring_whitespace();
                        angle
                    } else {
                        DEFAULT_OBLIQUE_ANGLE
                    }
                } else {
                    DEFAULT_OBLIQUE_ANGLE
                };

                if !(-90. ..=90.).contains(&angle.as_degrees()) {
                    return Err(ParseError);
                }

                Self::Oblique(angle)
            },
            _ => return Err(ParseError),
        };

        Ok(font_style)
    }
}

impl ToComputedStyle for FontStyle {
    type Computed = computed::FontStyle;

    fn to_computed_style(&self, context: &StyleContext) -> Self::Computed {
        _ = context;

        *self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::css::Origin;

    #[test]
    fn parse_font_style() {
        assert_eq!(
            FontStyle::parse_complete(&mut Parser::new("normal", Origin::Author)),
            Ok(FontStyle::Normal)
        );

        assert_eq!(
            FontStyle::parse_complete(&mut Parser::new("italic", Origin::Author)),
            Ok(FontStyle::Italic)
        );

        assert_eq!(
            FontStyle::parse_complete(&mut Parser::new("oblique", Origin::Author)),
            Ok(FontStyle::Oblique(DEFAULT_OBLIQUE_ANGLE))
        );

        assert_eq!(
            FontStyle::parse_complete(&mut Parser::new("oblique 10deg", Origin::Author)),
            Ok(FontStyle::Oblique(Angle::from_degrees(10.)))
        );
    }
}
