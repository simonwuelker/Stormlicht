use crate::{
    css::{self, syntax::Token, CSSParse},
    static_interned, InternedString,
};

/// <https://drafts.csswg.org/css-fonts/#font-family-prop>
#[derive(Clone, Debug)]
pub struct FontFamily {
    _desired_fonts: Vec<DesiredFont>,
}

#[derive(Clone, Debug)]
enum DesiredFont {
    /// <https://drafts.csswg.org/css-fonts/#family-name-syntax>
    FamilyName(InternedString),
    GenericFamily(GenericFontFamily),
}

/// <https://drafts.csswg.org/css-fonts/#generic-family-value>
#[derive(Clone, Copy, Debug)]
enum GenericFontFamily {
    /// <https://drafts.csswg.org/css-fonts/#serif-def>
    Serif,

    /// <https://drafts.csswg.org/css-fonts/#sans-serif-def>
    SansSerif,

    /// <https://drafts.csswg.org/css-fonts/#cursive-def>
    Cursive,

    /// <https://drafts.csswg.org/css-fonts/#fantasy-def>
    Fantasy,

    /// <https://drafts.csswg.org/css-fonts/#monospace-def>
    Monospace,

    /// <https://drafts.csswg.org/css-fonts/#system-ui-def>
    SystemUi,

    /// <https://drafts.csswg.org/css-fonts/#emoji-def>
    Emoji,

    /// <https://drafts.csswg.org/css-fonts/#math-def>
    Math,

    /// <https://drafts.csswg.org/css-fonts/#generic(fangsong)-def>
    GenericFangSong,

    /// <https://drafts.csswg.org/css-fonts/#ui-serif-def>
    UiSerif,

    /// <https://drafts.csswg.org/css-fonts/#ui-sans-serif-def>
    UiSansSerif,

    /// <https://drafts.csswg.org/css-fonts/#ui-monospace-def>
    UiMonospace,

    /// <https://drafts.csswg.org/css-fonts/#ui-rounded-def>
    UiRounded,
}

impl Default for FontFamily {
    fn default() -> Self {
        // The initial value is UA dependent
        Self {
            _desired_fonts: vec![DesiredFont::GenericFamily(GenericFontFamily::SansSerif)],
        }
    }
}

impl<'a> CSSParse<'a> for FontFamily {
    fn parse(parser: &mut css::Parser<'a>) -> Result<Self, css::ParseError> {
        let mut desired_fonts = vec![];

        while let Some(desired_font) = parser.parse_optional_value(CSSParse::parse) {
            desired_fonts.push(desired_font);
        }

        if desired_fonts.is_empty() {
            return Err(css::ParseError);
        }

        Ok(Self {
            _desired_fonts: desired_fonts,
        })
    }
}

impl<'a> CSSParse<'a> for DesiredFont {
    fn parse(parser: &mut css::Parser<'a>) -> Result<Self, css::ParseError> {
        if let Some(Token::String(name)) = parser.peek_token() {
            parser.next_token();
            Ok(Self::FamilyName(name))
        } else {
            let generic_family = CSSParse::parse(parser)?;
            Ok(Self::GenericFamily(generic_family))
        }
    }
}

impl<'a> CSSParse<'a> for GenericFontFamily {
    fn parse(parser: &mut css::Parser<'a>) -> Result<Self, css::ParseError> {
        let parsed_value = match parser.next_token() {
            Some(Token::Ident(static_interned!("serif"))) => Self::Serif,
            Some(Token::Ident(static_interned!("sans-serif"))) => Self::SansSerif,
            Some(Token::Ident(static_interned!("cursive"))) => Self::Cursive,
            Some(Token::Ident(static_interned!("fantasy"))) => Self::Fantasy,
            Some(Token::Ident(static_interned!("monospace"))) => Self::Monospace,
            Some(Token::Ident(static_interned!("system-ui"))) => Self::SystemUi,
            Some(Token::Ident(static_interned!("emoji"))) => Self::Emoji,
            Some(Token::Ident(static_interned!("math"))) => Self::Math,
            Some(Token::Ident(static_interned!("generic(fangsong)"))) => Self::GenericFangSong,
            Some(Token::Ident(static_interned!("ui-serif"))) => Self::UiSerif,
            Some(Token::Ident(static_interned!("ui-sans-serif"))) => Self::UiSansSerif,
            Some(Token::Ident(static_interned!("ui-monospace"))) => Self::UiMonospace,
            Some(Token::Ident(static_interned!("ui-rounded"))) => Self::UiRounded,
            _ => return Err(css::ParseError),
        };
        Ok(parsed_value)
    }
}
