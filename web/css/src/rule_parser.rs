use crate::{
    parser::{CSSParse, ParseError, Parser},
    selectors::SelectorList,
};

#[derive(Clone, Copy, Debug)]
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
        // FIXME: actually parse the rule body here
        while parser.next_token().is_some() {}
        Ok(ParsedRule { selectors })
    }
}

#[derive(Clone, Debug)]
pub struct ParsedRule<'a> {
    pub selectors: SelectorList<'a>,
}
