use crate::css::{
    selectors::Selector, syntax::Token, CSSParse, ParseError, Parser, StylePropertyDeclaration,
};

/// Used to track state across an CSS Stylesheet.
///
/// A parser should repeatedly call the methods on a [RuleParser]
/// so it can for example declare the Stylesheet as invalid if there's
/// a `@import` rule after another At-Rule that isn't `@charset`.
#[derive(Clone, Copy, Debug, Default)]
pub struct RuleParser;

impl RuleParser {
    pub fn parse_qualified_rule_prelude(
        &mut self,
        parser: &mut Parser<'_>,
    ) -> Result<Vec<Selector>, ParseError> {
        let selectors = parser.parse_comma_seperated_list(Selector::parse);

        if selectors.is_empty() {
            return Err(ParseError);
        }

        Ok(selectors)
    }

    pub fn parse_qualified_rule_block(
        &mut self,
        parser: &mut Parser<'_>,
    ) -> Result<Vec<StylePropertyDeclaration>, ParseError> {
        let mut properties = vec![];
        while !matches!(
            parser.peek_token_ignoring_whitespace(0),
            Some(Token::CurlyBraceClose)
        ) {
            if let Some(declaration) = parser.consume_declaration() {
                properties.push(declaration);

                if parser.expect_token(Token::Semicolon).is_err() {
                    // If this is not the last property in the rule body, this is a parse error!
                    parser.expect_exhausted()?;
                    break;
                }
            }
        }

        Ok(properties)
    }
}
