//! <https://drafts.csswg.org/css-color>

use crate::css::{
    parser::{CSSParse, ParseError, Parser},
    tokenizer::Token,
};

use super::Number;

/// <https://drafts.csswg.org/css-color/#color-syntax>
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
}

impl Color {
    pub fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self::rgba(red, green, blue, u8::MAX)
    }

    pub fn rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    fn parse_as_hex_color(parser: &mut Parser) -> Result<Self, ParseError> {
        // TODO: should we care about the hash flag here?
        if let Some(Token::Hash(ident, _)) = parser.next_token() {
            if ident.len() == 6 {
                // 6-digit hex number
                Ok(Self {
                    red: u8::from_str_radix(&ident[0..2], 16).map_err(|_| ParseError)?,
                    green: u8::from_str_radix(&ident[2..4], 16).map_err(|_| ParseError)?,
                    blue: u8::from_str_radix(&ident[4..6], 16).map_err(|_| ParseError)?,
                    alpha: u8::MAX,
                })
            } else if ident.len() == 8 {
                // 8-digit hex with alpha
                Ok(Self {
                    red: u8::from_str_radix(&ident[0..2], 16).map_err(|_| ParseError)?,
                    green: u8::from_str_radix(&ident[2..4], 16).map_err(|_| ParseError)?,
                    blue: u8::from_str_radix(&ident[4..6], 16).map_err(|_| ParseError)?,
                    alpha: u8::from_str_radix(&ident[6..8], 16).map_err(|_| ParseError)?,
                })
            } else if ident.len() == 3 {
                // Shorter version of 6-digit hex, each digit is "duplicated"
                Ok(Self {
                    red: u8::from_str_radix(&ident[0..1], 16).map_err(|_| ParseError)? * 0x11,
                    green: u8::from_str_radix(&ident[1..2], 16).map_err(|_| ParseError)? * 0x11,
                    blue: u8::from_str_radix(&ident[2..3], 16).map_err(|_| ParseError)? * 0x11,
                    alpha: u8::MAX,
                })
            } else if ident.len() == 4 {
                Ok(Self {
                    red: u8::from_str_radix(&ident[0..1], 16).map_err(|_| ParseError)? * 0x11,
                    green: u8::from_str_radix(&ident[1..2], 16).map_err(|_| ParseError)? * 0x11,
                    blue: u8::from_str_radix(&ident[2..3], 16).map_err(|_| ParseError)? * 0x11,
                    alpha: u8::from_str_radix(&ident[3..4], 16).map_err(|_| ParseError)? * 0x11,
                })
            } else {
                // Invalid length
                Err(ParseError)
            }
        } else {
            Err(ParseError)
        }
    }

    fn parse_legacy_rgb(parser: &mut Parser) -> Result<Self, ParseError> {
        // NOTE: The spec defines legacy-rgb and legacy-rgba
        //       But they are identical, so we do not differentiate between them
        let red = resolve_percentage(parser.expect_percentage()?);

        parser.skip_whitespace();
        parser.expect_token(Token::Comma)?;
        parser.skip_whitespace();

        let green = resolve_percentage(parser.expect_percentage()?);

        parser.skip_whitespace();
        parser.expect_token(Token::Comma)?;
        parser.skip_whitespace();

        let blue = resolve_percentage(parser.expect_percentage()?);

        parser.skip_whitespace();

        let alpha = parser
            .parse_optional_value(|p| {
                p.expect_token(Token::Comma)?;
                p.skip_whitespace();
                parse_alpha_value(p)
            })
            .unwrap_or(u8::MAX);
        parser.skip_whitespace();

        Ok(Self {
            red,
            green,
            blue,
            alpha,
        })
    }

    fn parse_modern_rgb(_parser: &mut Parser) -> Result<Self, ParseError> {
        // NOTE: The spec defines modern-rgb and modern-rgba
        //       But they are identical, so we do not differentiate between them
        todo!()
    }

    fn parse_rgb_function(parser: &mut Parser) -> Result<Self, ParseError> {
        if let Some(Token::Function(function_identifier)) = parser.next_token() {
            if function_identifier != "rgb" && function_identifier != "rgba" {
                return Err(ParseError);
            }

            if let Some(color) = parser.parse_optional_value(Self::parse_legacy_rgb) {
                return Ok(color);
            }

            Self::parse_modern_rgb(parser)
        } else {
            Err(ParseError)
        }
    }
}

impl<'a> CSSParse<'a> for Color {
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        if let Some(color) = parser.parse_optional_value(Self::parse_as_hex_color) {
            return Ok(color);
        }

        if let Some(color) = parser.parse_optional_value(Self::parse_rgb_function) {
            return Ok(color);
        }
        Err(ParseError)
    }
}

fn parse_alpha_value(parser: &mut Parser) -> Result<u8, ParseError> {
    let alpha = match parser.next_token() {
        Some(Token::Number(n)) => n.round_to_int().clamp(0, 255) as u8,
        Some(Token::Percentage(p)) => resolve_percentage(p),
        _ => return Err(ParseError),
    };
    parser.skip_whitespace();
    Ok(alpha)
}

fn resolve_percentage(percentage: Number) -> u8 {
    let clamped_percent = match percentage {
        Number::Number(f) => f.clamp(0., 100.),
        Number::Integer(i) => i.clamp(0, 100) as f32,
    };
    (clamped_percent * 2.55).round() as u8
}

#[cfg(test)]
mod tests {
    use super::Color;
    use crate::css::parser::{CSSParse, Parser};

    #[test]
    fn test_hex_color_code() {
        let mut six_digit_parser = Parser::new("#F00f10");
        assert_eq!(
            Color::parse_complete(&mut six_digit_parser),
            Ok(Color::rgb(0xF0, 0x0F, 0x10))
        );

        let mut eight_digit_parser = Parser::new("#F00f10AB");
        assert_eq!(
            Color::parse_complete(&mut eight_digit_parser),
            Ok(Color::rgba(0xF0, 0x0F, 0x10, 0xAB))
        );

        let mut six_digit_parser = Parser::new("#abc");
        assert_eq!(
            Color::parse_complete(&mut six_digit_parser),
            Ok(Color::rgb(0xAA, 0xBB, 0xCC))
        );

        let mut six_digit_parser = Parser::new("#abcd");
        assert_eq!(
            Color::parse_complete(&mut six_digit_parser),
            Ok(Color::rgba(0xAA, 0xBB, 0xCC, 0xDD))
        );
    }

    #[test]
    fn test_legacy_rgb() {
        let mut legacy_rgb = Parser::new("rgb(100%, 50.0%, 10%");
        assert_eq!(
            Color::parse_complete(&mut legacy_rgb),
            Ok(Color::rgb(255, 128, 26))
        );

        let mut legacy_rgba_number = Parser::new("rgb(100%, 50.0%, 10%, 1");
        assert_eq!(
            Color::parse_complete(&mut legacy_rgba_number),
            Ok(Color::rgba(255, 128, 26, 1))
        );

        let mut legacy_rgba_percent = Parser::new("rgb(100%, 50.0%, 10%, 1%");
        assert_eq!(
            Color::parse_complete(&mut legacy_rgba_percent),
            Ok(Color::rgba(255, 128, 26, 3))
        );
    }
}
