use crate::css::{selectors::SelectorList, CSSParse, ParseError, Parser, StyleProperty};

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
        let properties = vec![];
        // FIXME: actually parse the rule body here
        while parser.next_token().is_some() {}
        Ok(ParsedRule {
            selectors,
            properties,
        })
    }
}

#[derive(Clone, Debug)]
pub struct ParsedRule<'a> {
    pub selectors: SelectorList<'a>,
    properties: Vec<StyleProperty>,
}

impl<'a> ParsedRule<'a> {
    #[must_use]
    pub fn properties(&self) -> &[StyleProperty] {
        &self.properties
    }
}
