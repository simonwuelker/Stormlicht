use crate::css::{
    selectors::SelectorList, syntax::Token, CSSParse, ParseError, Parser, StylePropertyDeclaration,
};

/// Used to track state across an CSS Stylesheet.
///
/// A parser should repeatedly call the methods on a [RuleParser]
/// so it can for example declare the Stylesheet as invalid if there's
/// a `@import` rule after another At-Rule that isn't `@charset`.
#[derive(Clone, Copy, Debug, Default)]
pub struct RuleParser;

impl RuleParser {
    pub fn parse_qualified_rule_prelude<'a>(
        &self,
        parser: &mut Parser<'a>,
    ) -> Result<SelectorList<'a>, ParseError> {
        SelectorList::parse_complete(parser)
    }

    pub fn parse_qualified_rule_block<'a>(
        &self,
        parser: &mut Parser<'a>,
        selectors: SelectorList<'a>,
    ) -> Result<ParsedRule<'a>, ParseError> {
        let mut properties = vec![];
        while !parser.is_exhausted() {
            if let Some(declaration) = parser.consume_declaration() {
                properties.push(declaration);
            }

            if parser.expect_token(Token::Semicolon).is_err() {
                // If this is not the last property in the rule body, this is a parse error!
                parser.skip_whitespace();
                parser.expect_exhausted()?;
                break;
            }
            parser.skip_whitespace();
        }

        Ok(ParsedRule {
            selectors,
            properties,
        })
    }
}

#[derive(Clone, Debug)]
pub struct ParsedRule<'a> {
    pub selectors: SelectorList<'a>,
    properties: Vec<StylePropertyDeclaration>,
}

impl<'a> ParsedRule<'a> {
    #[must_use]
    pub fn properties(&self) -> &[StylePropertyDeclaration] {
        &self.properties
    }
}
