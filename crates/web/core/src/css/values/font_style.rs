use crate::{
    css::{values::Angle, CSSParse, ParseError, Parser},
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
                parser.skip_whitespace();

                let angle: Angle = parser.parse_optional().unwrap_or(DEFAULT_OBLIQUE_ANGLE);

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
