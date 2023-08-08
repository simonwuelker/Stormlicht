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

use string_interner::{static_interned, static_str};

use super::{
    rule_parser::RuleParser,
    tokenizer::{Token, Tokenizer},
};

use crate::css::{
    properties::Important, values::Number, Origin, StyleProperty, StylePropertyDeclaration,
    StyleRule, Stylesheet,
};
use std::fmt::Debug;

const MAX_ITERATIONS: usize = 128;

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
    buffered_token: Option<Token>,
    stop_at: Option<ParserDelimiter>,
    stopped: bool,
    origin: Origin,
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
pub struct ParserState {
    pub position: usize,
    buffered_token: Option<Token>,
    stopped: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParseError;

impl<'a> Parser<'a> {
    pub fn new(source: &'a str, origin: Origin) -> Self {
        let mut parser = Self {
            tokenizer: Tokenizer::new(source),
            buffered_token: None,
            stop_at: None,
            stopped: false,
            origin,
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
            origin: self.origin,
        }
    }

    pub fn next_token(&mut self) -> Option<Token> {
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

    pub fn peek_token(&mut self) -> Option<Token> {
        let next_token = self.next_token()?;
        self.reconsume(next_token.clone());
        Some(next_token)
    }

    #[inline]
    pub fn expect_token(&mut self, expected_token: Token) -> Result<(), ParseError> {
        if self.next_token() == Some(expected_token) {
            Ok(())
        } else {
            Err(ParseError)
        }
    }

    #[inline]
    pub fn state(&self) -> ParserState {
        ParserState {
            position: self.tokenizer.position(),
            buffered_token: self.buffered_token.clone(),
            stopped: self.stopped,
        }
    }

    #[inline]
    pub fn set_state(&mut self, state: ParserState) {
        self.tokenizer.set_position(state.position);
        self.buffered_token = state.buffered_token;
        self.stopped = state.stopped;
    }

    /// <https://drafts.csswg.org/css-syntax/#reconsume-the-current-input-token>
    #[inline]
    fn reconsume(&mut self, token: Token) {
        assert!(self.buffered_token.is_none());
        self.buffered_token = Some(token);
    }

    /// <https://drafts.csswg.org/css-syntax/#consume-list-of-rules>
    pub fn consume_list_of_rules(
        &mut self,
        rule_parser: &mut RuleParser,
        top_level: TopLevel,
    ) -> Vec<StyleRule> {
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
    ) -> Result<StyleRule, ParseError> {
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
        self.skip_whitespace();

        // Create a delimited parser that consumes the rule's block
        let mut block_parser = self.create_limited(ParserDelimiter::CURLY_BRACE_CLOSE);
        let qualified_rule =
            rule_parser.parse_qualified_rule_block(&mut block_parser, selectors)?;
        block_parser.expect_exhausted()?;

        self.set_state(block_parser.state());
        self.expect_token(Token::CurlyBraceClose)?;
        self.skip_whitespace();

        Ok(qualified_rule)
    }

    /// <https://drafts.csswg.org/css-syntax-3/#consume-declaration>
    pub fn consume_declaration(&mut self) -> Option<StylePropertyDeclaration> {
        self.consume_declaration_with_nested(false)
    }

    /// <https://drafts.csswg.org/css-syntax-3/#consume-declaration>
    fn consume_declaration_with_nested(
        &mut self,
        nested: bool,
    ) -> Option<StylePropertyDeclaration> {
        // Let decl be a new declaration, with an initially empty name and a value set to an empty list.
        // NOTE: We don't construct declarations like this.
        let mut important = Important::No;

        // 1. If the next token is an <ident-token>, consume a token from input and set decl’s name to the token’s value.
        //    Otherwise, consume the remnants of a bad declaration from input, with nested, and return nothing.
        let declaration_name = if let Some(Token::Ident(name)) = self.peek_token() {
            self.next_token();
            name
        } else {
            self.consume_remnants_of_bad_declaration(nested);
            return None;
        };

        // 2. Discard whitespace from input.
        self.skip_whitespace();

        // 3. If the next token is a <colon-token>, discard a token from input.
        //    Otherwise, consume the remnants of a bad declaration from input, with nested, and return nothing.
        if let Some(Token::Colon) = self.peek_token() {
            self.next_token();
        } else {
            self.consume_remnants_of_bad_declaration(nested);
            return None;
        }

        // 4. Discard whitespace from input.
        self.skip_whitespace();

        // NOTE: At this point we deviate from the spec because the spec gets a little silly
        let value = if let Ok(value) = StyleProperty::parse_value(self, declaration_name) {
            value
        } else {
            self.consume_remnants_of_bad_declaration(nested);
            return None;
        };

        // Check for !important
        if matches!(self.peek_token(), Some(Token::Delim('!'))) {
            self.next_token();

            #[allow(clippy::redundant_guards)] // In this case, the guard helps with readability
            match self.next_token() {
                Some(Token::Ident(i)) if i == static_interned!("important") => {
                    important = Important::Yes;
                    self.skip_whitespace();
                },
                _ => {
                    self.consume_remnants_of_bad_declaration(nested);
                    return None;
                },
            }
        }

        Some(StylePropertyDeclaration { value, important })
    }

    /// <https://drafts.csswg.org/css-syntax-3/#consume-the-remnants-of-a-bad-declaration>
    fn consume_remnants_of_bad_declaration(&mut self, nested: bool) {
        _ = nested;
        // NOTE: This is not what the spec does.
        // But for now, it should be more or less equivalent (we don't respect "}")
        // Process input:
        while !matches!(self.next_token(), Some(Token::Semicolon) | None) {}
    }

    pub fn parse_stylesheet(&mut self) -> Result<Stylesheet, ParseError> {
        // NOTE: The ruleparser shouldn't stay a unit struct
        #[allow(clippy::default_constructed_unit_structs)]
        let mut rule_parser = RuleParser::default();

        let mut rules = vec![];

        while let Ok(rule) =
            self.consume_qualified_rule(&mut rule_parser, MixedWithDeclarations::No)
        {
            rules.push(rule);
        }

        Ok(Stylesheet::new(self.origin, rules))
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
        let mut iterations = 0;

        while let Ok(parsed_value) = closure(self) {
            if iterations == MAX_ITERATIONS {
                log::warn!("Exceeded maximum number of iterations, skipping...");
                break;
            }

            parsed_tokens.push(parsed_value);
            state_before_last_token = self.state();

            self.skip_whitespace();
            if !matches!(self.next_token(), Some(Token::Comma)) {
                break;
            }
            self.skip_whitespace();

            iterations += 1;
        }

        // Don't consume the token that caused us to exit the loop
        self.set_state(state_before_last_token);

        // But consume the eventual whitespace after it :)
        self.skip_whitespace();

        parsed_tokens
    }

    /// Apply a parser, but fail if the reader state is not advanced.
    ///
    /// # Specification
    /// <https://drafts.csswg.org/css-values-4/#mult-req>
    pub fn parse_nonempty<T: Debug, F>(&mut self, closure: F) -> Result<T, ParseError>
    where
        F: Fn(&mut Self) -> Result<T, ParseError>,
    {
        // Remember where we were at before we parsed a list
        let position = self.tokenizer.position();
        let has_token_buffered = self.buffered_token.is_some();

        // Apply the parser
        let parsed_token = closure(self)?;

        // Fail if our reader was not advanced
        if self.tokenizer.position() == position
            && self.buffered_token.is_some() == has_token_buffered
        {
            Err(ParseError)
        } else {
            Ok(parsed_token)
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
        let mut iterations = 0;

        while let Ok(parsed_value) = closure(self) {
            if iterations == MAX_ITERATIONS {
                log::warn!("Exceeded maximum number of iterations, skipping...");
                break;
            }

            state_before_end_token = self.state();
            parsed_tokens.push(parsed_value);

            if whitespace_allowed == WhitespaceAllowed::Yes {
                self.skip_whitespace();
            }

            iterations += 1;
        }

        // Reset to the last valid state to avoid accidentally consuming too many tokens
        self.set_state(state_before_end_token);

        if whitespace_allowed == WhitespaceAllowed::Yes {
            self.skip_whitespace();
        }
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

    pub fn is_exhausted(&mut self) -> bool {
        self.peek_token().is_none()
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
    ///
    /// This function is primarily intended to be used in tests.
    #[cfg(test)]
    fn parse_from_str(source: &'a str) -> Result<Self, ParseError> {
        let mut parser = Parser::new(source, Origin::Author);
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

impl<'a, T: CSSParse<'a> + Debug> CSSParse<'a> for Option<T> {
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        Ok(parser.parse_optional_value(T::parse))
    }
}
