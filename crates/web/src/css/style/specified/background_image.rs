//! <https://drafts.csswg.org/css-backgrounds/#background-image>

use crate::{
    css::{
        style::{StyleContext, ToComputedStyle},
        syntax::Token,
        values::Url,
        CSSParse, ParseError, Parser,
    },
    static_interned,
};

/// <https://drafts.csswg.org/css-backgrounds/#background-image>
#[derive(Clone, Debug)]
pub struct BackgroundImage {
    // TODO: The spec explicitly treats the "none" layers as layers (that are not rendered).
    //       Is this necessary? Do we need to keep these around?
    layers: Vec<Option<Url>>,
}

impl BackgroundImage {
    #[must_use]
    pub fn layers(&self) -> &[Option<Url>] {
        &self.layers
    }
}

impl<'a> CSSParse<'a> for BackgroundImage {
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let first_layer = parse_single_layer(parser)?;

        let mut layers = vec![first_layer];
        while let Some(Token::Comma) = parser.peek_token_ignoring_whitespace(0) {
            let _ = parser.next_token_ignoring_whitespace();

            let layer = parse_single_layer(parser)?;
            layers.push(layer);
        }

        Ok(Self { layers })
    }
}

fn parse_single_layer(parser: &mut Parser<'_>) -> Result<Option<Url>, ParseError> {
    let parsed_layer = match parser.peek_token_ignoring_whitespace(0) {
        Some(Token::Ident(ident)) if *ident == static_interned!("none") => {
            let _ = parser.next_token_ignoring_whitespace();
            None
        },
        _ => {
            let url = Url::parse(parser)?;
            Some(url)
        },
    };

    Ok(parsed_layer)
}

impl Default for BackgroundImage {
    fn default() -> Self {
        Self { layers: vec![None] }
    }
}

impl ToComputedStyle for BackgroundImage {
    type Computed = Self;

    fn to_computed_style(&self, context: &StyleContext) -> Self::Computed {
        _ = context;

        self.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fail_to_parse_background_image_with_no_layers() {
        assert!(BackgroundImage::parse_from_str("").is_err());
    }
    #[test]
    fn parse_background_image_url() {
        let bg_image = BackgroundImage::parse_from_str("url(foobar)").unwrap();
        let layers = bg_image.layers();

        assert!(layers.len() == 1);
        assert!(layers[0].is_some());
    }

    #[test]
    fn parse_background_image_none() {
        let bg_image = BackgroundImage::parse_from_str("none").unwrap();
        let layers = bg_image.layers();

        assert!(layers.len() == 1);
        assert!(layers[0].is_none());
    }

    #[test]
    fn parse_multiple_background_layers() {
        let bg_image = BackgroundImage::parse_from_str("none, url(test), none").unwrap();
        let layers = bg_image.layers();

        assert!(layers.len() == 3);
        assert!(layers[0].is_none());
        assert!(layers[1].is_some());
        assert!(layers[2].is_none());
    }
}
