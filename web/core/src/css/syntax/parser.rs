//! Higher-level parsing functions.
//!
//! This is the next parsing stage after [Tokenization](crate::tokenizer).
//!
//! # Parsing Rules
//! ## Error handling
//! Parser functions usually return `Result<T, ParseError>`. If the `Result` is `Ok`,
//! the parser will have consumed *exactly* the expected amount of tokens.
//! If `Err` is returned, the state of the parser is undefined. Callers that want to handle
//! optional values should backup the state beforehand with [Parser::state] and then set it
//! after parsing with [Parser::set_state].
//!
//! ## Whitespace
//! The term "whitespace" includes comments.
//! Any parsing function should consume any trailing whitespace *after* it's input but not *before it*.

use super::{
    rule_parser::{ParsedRule, RuleParser},
    tokenizer::{Token, Tokenizer},
};

use crate::css::values::Number;
use std::fmt::Debug;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum MixedWithDeclarations {
    Yes,
    #[default]
    No,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TopLevel {
    Yes,
    #[default]
    No,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum WhitespaceAllowed {
    #[default]
    Yes,
    No,
}
#[derive(Clone, Debug)]
pub struct Parser<'a> {
    tokenizer: Tokenizer<'a>,
    buffered_token: Option<Token<'a>>,
    stop_at: Option<ParserDelimiter>,
    stopped: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct ParserDelimiter(u8);

impl ParserDelimiter {
    const CURLY_BRACE_OPEN: Self = Self(0b00000001);
    const CURLY_BRACE_CLOSE: Self = Self(0b00000010);
    const SEMICOLON: Self = Self(0b00000100);

    #[must_use]
    pub fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    #[must_use]
    pub fn or(&self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

#[derive(Clone, Debug)]
pub struct ParserState<'a> {
    position: usize,
    buffered_token: Option<Token<'a>>,
    stopped: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParseError;

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut parser = Self {
            tokenizer: Tokenizer::new(source),
            buffered_token: None,
            stop_at: None,
            stopped: false,
        };
        parser.skip_whitespace();
        parser
    }

    /// Create a parser from the same source that stops once it reaches a certain token.
    ///
    /// The final delimited parser position will be right **before** the delimiter token.
    pub fn create_limited(&self, limit: ParserDelimiter) -> Self {
        Self {
            tokenizer: self.tokenizer,
            buffered_token: self.buffered_token.clone(),
            stop_at: Some(limit),
            stopped: false,
        }
    }

    #[inline]
    pub fn next_token(&mut self) -> Option<Token<'a>> {
        if self.stopped {
            return None;
        }

        let position_before_token = self.state();
        let next_token = self
            .buffered_token
            .take()
            .or_else(|| self.tokenizer.next_token())?;

        if let Some(stop_at) = self.stop_at {
            let should_stop = match next_token {
                Token::CurlyBraceOpen => stop_at.contains(ParserDelimiter::CURLY_BRACE_OPEN),
                Token::CurlyBraceClose => stop_at.contains(ParserDelimiter::CURLY_BRACE_CLOSE),
                Token::Semicolon => stop_at.contains(ParserDelimiter::SEMICOLON),
                _ => false,
            };
            if should_stop {
                self.stopped = true;
                self.set_state(position_before_token);
                return None;
            }
        }

        Some(next_token)
    }

    pub fn peek_token(&mut self) -> Option<Token<'a>> {
        let next_token = self.next_token()?;
        self.reconsume(next_token.clone());
        Some(next_token)
    }

    #[inline]
    pub fn expect_token(&mut self, expected_token: Token<'a>) -> Result<(), ParseError> {
        if self.next_token() == Some(expected_token) {
            Ok(())
        } else {
            Err(ParseError)
        }
    }

    #[inline]
    pub fn state(&self) -> ParserState<'a> {
        ParserState {
            position: self.tokenizer.position(),
            buffered_token: self.buffered_token.clone(),
            stopped: self.stopped,
        }
    }

    #[inline]
    pub fn set_state(&mut self, state: ParserState<'a>) {
        self.tokenizer.set_position(state.position);
        self.buffered_token = state.buffered_token;
        self.stopped = state.stopped;
    }

    /// <https://drafts.csswg.org/css-syntax/#reconsume-the-current-input-token>
    #[inline]
    fn reconsume(&mut self, token: Token<'a>) {
        assert!(self.buffered_token.is_none());
        self.buffered_token = Some(token);
    }

    /// <https://drafts.csswg.org/css-syntax/#consume-list-of-rules>
    pub fn consume_list_of_rules(
        &mut self,
        rule_parser: &mut RuleParser,
        top_level: TopLevel,
    ) -> Vec<ParsedRule<'a>> {
        // Create an initially empty list of rules.
        let mut rules = vec![];

        // Repeatedly consume the next input token:
        loop {
            match self.next_token() {
                Some(Token::Whitespace) => {
                    // Do nothing.
                },
                None => {
                    // Return the list of rules.
                    return rules;
                },
                Some(token @ (Token::CommentDeclarationOpen | Token::CommentDeclarationClose)) => {
                    // If the top-level flag is set,
                    if top_level == TopLevel::Yes {
                        // do nothing.
                    }
                    // Otherwise,
                    else {
                        //  reconsume the current input token
                        self.reconsume(token);

                        // Consume a qualified rule
                        // If anything is returned, append it to the list of rules.
                        if let Ok(rule) = self
                            .consume_qualified_rule(rule_parser, MixedWithDeclarations::default())
                        {
                            rules.push(rule);
                        }
                    }
                },
                Some(token @ Token::AtKeyword(_)) => {
                    // Reconsume the current input token
                    self.reconsume(token);

                    // Consume an at-rule, and append the returned value to the list of rules.
                    // rules.push(self.consume_at_rule());
                },
                Some(other_token) => {
                    // Reconsume the current input token
                    self.reconsume(other_token);

                    // Consume a qualified rule. If anything is returned, append it to the list of rules.
                    if let Ok(rule) =
                        self.consume_qualified_rule(rule_parser, MixedWithDeclarations::default())
                    {
                        rules.push(rule);
                    }
                },
            }
        }
    }

    /// <https://drafts.csswg.org/css-syntax/#consume-a-qualified-rule>
    pub fn consume_qualified_rule(
        &mut self,
        rule_parser: &mut RuleParser,
        mixed_with_declarations: MixedWithDeclarations,
    ) -> Result<ParsedRule<'a>, ParseError> {
        // NOTE: The spec sometimes returns "None" (not an error, but no rule.)
        // Since "None" and "Err(_)" are treated the same (the parser ignores the rule and moves on),
        // we never return None, always Err(_).

        // Create a delimited parser that only consumes the rule's prelude
        let prelude_ends_at = if mixed_with_declarations == MixedWithDeclarations::Yes {
            ParserDelimiter::CURLY_BRACE_OPEN.or(ParserDelimiter::SEMICOLON)
        } else {
            ParserDelimiter::CURLY_BRACE_OPEN
        };

        let mut prelude_parser = self.create_limited(prelude_ends_at);
        let selectors = rule_parser.parse_qualified_rule_prelude(&mut prelude_parser)?;
        prelude_parser.expect_exhausted()?;

        self.set_state(prelude_parser.state());
        self.expect_token(Token::CurlyBraceOpen)?; // FIXME: this could be a semicolon

        // Create a delimited parser that consumes the rule's block
        let mut block_parser = self.create_limited(ParserDelimiter::CURLY_BRACE_CLOSE);
        let qualified_rule =
            rule_parser.parse_qualified_rule_block(&mut block_parser, selectors)?;
        block_parser.expect_exhausted()?;
        self.set_state(block_parser.state());
        self.expect_token(Token::CurlyBraceClose)?;

        Ok(qualified_rule)
    }

    pub fn parse_stylesheet(&mut self) -> Result<Vec<ParsedRule<'a>>, ParseError> {
        // NOTE: The ruleparser shouldn't stay a unit struct
        #[allow(clippy::default_constructed_unit_structs)]
        let mut rule_parser = RuleParser::default();

        let mut rules = vec![];

        while let Ok(rule) =
            self.consume_qualified_rule(&mut rule_parser, MixedWithDeclarations::No)
        {
            rules.push(rule);
        }

        Ok(rules)
    }

    /// Applies a parser as often as possible, seperating individual parser calls by
    /// [Comma](Token::Comma) tokens (and optionally [whitespace](Token::Whitespace) or comments).
    /// It is possible that no tokens will be produced. If this is not desired, use
    /// [parse_nonempty_comma_seperated_list](Self::parse_nonempty_comma_seperated_list) instead.
    ///
    /// # Specification
    /// <https://w3c.github.io/csswg-drafts/css-values-4/#mult-comma>
    pub fn parse_comma_seperated_list<T: Debug, F>(&mut self, closure: F) -> Vec<T>
    where
        F: Fn(&mut Self) -> Result<T, ParseError>,
    {
        let mut parsed_tokens = vec![];
        let mut state_before_last_token = self.state();

        while let Ok(parsed_value) = closure(self) {
            parsed_tokens.push(parsed_value);
            state_before_last_token = self.state();

            self.skip_whitespace();
            if !matches!(self.next_token(), Some(Token::Comma)) {
                break;
            }
            self.skip_whitespace();
        }

        // Don't consume the token that caused us to exit the loop
        self.set_state(state_before_last_token);

        parsed_tokens
    }

    /// Applies a parser as often as possible, seperating individual parser calls by
    /// [Comma](Token::Comma) tokens (and optionally [whitespace](Token::Whitespace) or comments).
    /// The parsing fails if no tokens are produced. If this is not desired, use [parse_comma_seperated_list](Self::parse_comma_seperated_list) instead.
    ///
    /// # Specification
    /// <https://w3c.github.io/csswg-drafts/css-values-4/#mult-req>
    pub fn parse_nonempty_comma_seperated_list<T: Debug, F>(
        &mut self,
        closure: F,
    ) -> Result<Vec<T>, ParseError>
    where
        F: Fn(&mut Self) -> Result<T, ParseError>,
    {
        let parsed_tokens = self.parse_comma_seperated_list(closure);

        if parsed_tokens.is_empty() {
            Err(ParseError)
        } else {
            Ok(parsed_tokens)
        }
    }

    /// Applies a parser as often as possible, including (possibly) not at all.
    ///
    /// The `whitespace_allowed` parameter can be used to control if parser calls may be seperated
    /// by whitespace. If you are not sure whether they can be, it's generally okay to pass
    /// `WhitespaceAllowed::Yes`.
    ///
    /// # Specification
    /// <https://w3c.github.io/csswg-drafts/css-values-4/#mult-zero-plus>
    pub fn parse_any_number_of<T: Debug, F>(
        &mut self,
        closure: F,
        whitespace_allowed: WhitespaceAllowed,
    ) -> Vec<T>
    where
        F: Fn(&mut Self) -> Result<T, ParseError>,
    {
        let mut parsed_tokens = vec![];
        let mut state_before_end_token = self.state();
        while let Ok(parsed_value) = closure(self) {
            state_before_end_token = self.state();
            parsed_tokens.push(parsed_value);

            if whitespace_allowed == WhitespaceAllowed::Yes {
                self.skip_whitespace();
            }
        }

        // Reset to the last valid state to avoid accidentally consuming too many tokens
        self.set_state(state_before_end_token);
        parsed_tokens
    }

    /// Apply a parser but don't throw an error if parsing fails.
    /// If no value could be parsed, `None` is returned and the internal
    /// state is not advanced.
    ///
    /// # Specification
    /// <https://w3c.github.io/csswg-drafts/css-values-4/#mult-opt>
    pub fn parse_optional_value<T: Debug, F>(&mut self, closure: F) -> Option<T>
    where
        F: Fn(&mut Self) -> Result<T, ParseError>,
    {
        let state = self.state();
        let x = closure(self);
        match x {
            Ok(parsed_value) => Some(parsed_value),
            Err(_) => {
                self.set_state(state);
                None
            },
        }
    }

    /// Return an error if any tokens are left in the token stream.
    ///
    /// If `Err` is returned, the state of the parser is unspecified.
    pub fn expect_exhausted(&mut self) -> Result<(), ParseError> {
        if self.next_token().is_none() {
            Ok(())
        } else {
            Err(ParseError)
        }
    }

    /// Return an error if the next token is not a whitespace
    /// The whitespace is consumed.
    ///
    /// If `Err` is returned, the state of the parser is unspecified.
    pub fn expect_whitespace(&mut self) -> Result<(), ParseError> {
        if matches!(self.next_token(), Some(Token::Whitespace)) {
            Ok(())
        } else {
            Err(ParseError)
        }
    }

    /// Skip any potential whitespaces, possibly none.
    ///
    /// If you want to ensure that at least one whitespace exists, call
    /// [expect_whitespace](Parser::expect_whitespace) beforehand.
    pub fn skip_whitespace(&mut self) {
        let mut state_before_non_whitespace = self.state();

        // // FIXME: skip comments too
        while let Some(Token::Whitespace) = self.next_token() {
            state_before_non_whitespace = self.state();
        }

        self.set_state(state_before_non_whitespace);
    }

    pub fn expect_percentage(&mut self) -> Result<Number, ParseError> {
        if let Some(Token::Percentage(percentage)) = self.next_token() {
            Ok(percentage)
        } else {
            Err(ParseError)
        }
    }
}

/// Types that can be parsed from a [Parser]
pub trait CSSParse<'a>: Sized {
    /// Try to parse an instance of the type from CSS source code.
    ///
    /// If any tokens remain in the source after the instance is parsed, an
    /// error is returned.
    fn parse_from_str(source: &'a str) -> Result<Self, ParseError> {
        let mut parser = Parser::new(source);
        let parsed_value = Self::parse(&mut parser)?;
        parser.expect_exhausted()?;
        Ok(parsed_value)
    }

    /// Try to parse an instance of the type from the parse source.
    ///
    /// If `Ok` is returned, the parser will have consumed all the tokens that belonged to the instance (but not more).
    ///
    /// If `Err` is returned, the state of the parser is undefined.
    /// Callers that expect errors should therefore backup the parse state with [Parser::state]
    /// before calling this method.
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError>;

    fn parse_complete(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let parsed_value = Self::parse(parser)?;
        parser.expect_exhausted()?;
        Ok(parsed_value)
    }
}
