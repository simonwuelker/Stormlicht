//! Higher-level parsing functions.
//!
//! This is the next parsing stage after [Tokenization](super::tokenizer).

use sl_std::ring_buffer::RingBuffer;

use super::{
    rule_parser::RuleParser,
    tokenizer::{Token, Tokenizer},
};

use crate::{
    css::{
        layout::Sides, properties::Important, values::Number, Origin, StyleProperty,
        StylePropertyDeclaration, StyleRule, Stylesheet,
    },
    static_interned, InternedString,
};

use std::fmt::Debug;

const MAX_ITERATIONS: usize = 128;

/// The maximum number of tokens that can be peeked ahead during parsing
const MAX_LOOKAHEAD: usize = 16;

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

#[derive(Clone, Debug)]
pub struct Parser<'a> {
    tokenizer: Tokenizer<'a>,
    queued_tokens: RingBuffer<Token, MAX_LOOKAHEAD>,
    origin: Origin,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParseError;

impl<'a> Parser<'a> {
    #[must_use]
    pub fn new(source: &'a str, origin: Origin) -> Self {
        Self {
            tokenizer: Tokenizer::new(source),
            queued_tokens: RingBuffer::default(),
            origin,
        }
    }

    /// Make sure that there are at least `n` more tokens in the queue
    ///
    /// This collapses sequences of whitespace tokens, since they have no semantic meaning.
    ///
    /// The queue might not contain `n` elements afterwards if the end of file was reached.
    fn queue_tokens(&mut self, n: usize) {
        if n <= self.queued_tokens.len() {
            return;
        }

        let mut last_token_was_whitespace = self
            .queued_tokens
            .peek_back(0)
            .is_some_and(Token::is_whitespace);

        for _ in 0..n - self.queued_tokens.len() {
            while let Some(token) = self.tokenizer.next_token() {
                if token.is_whitespace() {
                    if last_token_was_whitespace {
                        continue;
                    } else {
                        last_token_was_whitespace = true;
                        self.queued_tokens.push(token);
                        break;
                    }
                } else {
                    self.queued_tokens.push(token);
                    break;
                }
            }
        }
    }

    #[must_use]
    pub fn next_token(&mut self) -> Option<Token> {
        self.queue_tokens(1);
        self.queued_tokens.pop_front()
    }

    #[must_use]
    pub fn next_token_ignoring_whitespace(&mut self) -> Option<Token> {
        let mut token = self.next_token()?;
        while token.is_whitespace() {
            token = self.next_token()?;
        }

        Some(token)
    }

    #[must_use]
    pub fn peek_token(&mut self, n: usize) -> Option<&Token> {
        self.queue_tokens(n + 1);
        self.queued_tokens.peek_front(n)
    }

    #[inline]
    #[must_use]
    pub fn peek_token_ignoring_whitespace(&mut self, n: usize) -> Option<&Token> {
        self.queue_tokens(2 * n + 2);

        let mut non_whitespace_tokens_seen = 0;
        let mut tokens_seen = 0;

        while let Some(token) = self.queued_tokens.peek_front(tokens_seen) {
            if !token.is_whitespace() {
                if non_whitespace_tokens_seen == n {
                    return Some(token);
                }
                non_whitespace_tokens_seen += 1;
            }
            tokens_seen += 1;
        }

        None
    }

    #[inline]
    pub fn expect_token(&mut self, expected_token: Token) -> Result<(), ParseError> {
        if self.next_token_ignoring_whitespace() == Some(expected_token) {
            Ok(())
        } else {
            Err(ParseError)
        }
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
            match self.peek_token_ignoring_whitespace(0) {
                None => {
                    // Return the list of rules.
                    return rules;
                },
                Some(Token::CommentDeclarationOpen | Token::CommentDeclarationClose) => {
                    // If the top-level flag is set,
                    if top_level == TopLevel::Yes {
                        // do nothing.
                        _ = self.next_token();
                    }
                    // Otherwise,
                    else {
                        // reconsume the current input token
                        // Consume a qualified rule
                        // If anything is returned, append it to the list of rules.
                        if let Ok(rule) = self
                            .consume_qualified_rule(rule_parser, MixedWithDeclarations::default())
                        {
                            rules.push(rule);
                        }
                    }
                },
                Some(Token::AtKeyword(_)) => {
                    // Reconsume the current input token

                    // Consume an at-rule, and append the returned value to the list of rules.
                    // rules.push(self.consume_at_rule());
                },
                Some(_) => {
                    // Reconsume the current input token
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
        _ = mixed_with_declarations;

        // Parse the rule prelude (selectors)
        let selectors = rule_parser.parse_qualified_rule_prelude(self)?;
        self.expect_token(Token::CurlyBraceOpen)?; // FIXME: this could be a semicolon

        // Parse the rule block
        let properties = rule_parser.parse_qualified_rule_block(self)?;
        let qualified_rule = StyleRule::new(selectors, properties);
        self.expect_token(Token::CurlyBraceClose)?;

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
        let declaration_name =
            if let Some(Token::Ident(name)) = self.peek_token_ignoring_whitespace(0) {
                let name = *name;
                let _ = self.next_token_ignoring_whitespace();
                name
            } else {
                self.consume_remnants_of_bad_declaration(nested);
                return None;
            };

        // 2. Discard whitespace from input.
        // 3. If the next token is a <colon-token>, discard a token from input.
        //    Otherwise, consume the remnants of a bad declaration from input, with nested, and return nothing.
        if let Some(Token::Colon) = self.peek_token_ignoring_whitespace(0) {
            let _ = self.next_token_ignoring_whitespace();
        } else {
            self.consume_remnants_of_bad_declaration(nested);
            return None;
        }

        // 4. Discard whitespace from input.

        // NOTE: At this point we deviate from the spec because the spec gets a little silly
        let value = if let Ok(value) = StyleProperty::parse_value(self, declaration_name) {
            value
        } else {
            self.consume_remnants_of_bad_declaration(nested);
            return None;
        };

        // Check for !important
        if matches!(
            self.peek_token_ignoring_whitespace(0),
            Some(Token::Delim('!'))
        ) {
            let _ = self.next_token_ignoring_whitespace();

            #[allow(clippy::redundant_guards)] // In this case, the guard helps with readability
            match self.next_token() {
                Some(Token::Ident(i)) if i == static_interned!("important") => {
                    important = Important::Yes;
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
    ///
    /// This returns the parser from anywhere within a rule that we can't parse to the beginning
    /// of the next rule
    fn consume_remnants_of_bad_declaration(&mut self, nested: bool) {
        _ = nested;
        // NOTE: This is not what the spec does.
        // But for now, it should be more or less equivalent (we don't respect "}")
        // Process input:
        loop {
            match self.peek_token_ignoring_whitespace(0) {
                Some(Token::Semicolon) => {
                    let _ = self.next_token_ignoring_whitespace();
                    break;
                },
                Some(Token::CurlyBraceClose) | None => break,
                _ => {
                    let _ = self.next_token_ignoring_whitespace();
                },
            }
        }
    }

    pub fn parse_stylesheet(&mut self, index: usize) -> Stylesheet {
        // NOTE: The ruleparser shouldn't stay a unit struct
        #[allow(clippy::default_constructed_unit_structs)]
        let mut rule_parser = RuleParser::default();

        let mut rules = vec![];

        while let Ok(rule) =
            self.consume_qualified_rule(&mut rule_parser, MixedWithDeclarations::No)
        {
            // There's no point in caring about empty rules, so let's drop them
            if !rule.properties().is_empty() {
                rules.push(rule);
            }
        }

        Stylesheet::new(self.origin, rules, index)
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
        let mut iterations = 0;

        while let Ok(parsed_value) = closure(self) {
            if iterations == MAX_ITERATIONS {
                log::warn!("Exceeded maximum number of iterations, skipping...");
                break;
            }

            parsed_tokens.push(parsed_value);

            if !matches!(self.peek_token_ignoring_whitespace(0), Some(Token::Comma)) {
                break;
            }
            _ = self.next_token_ignoring_whitespace();

            iterations += 1;
        }

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
        let position = self.tokenizer.get_position();
        let n_buffered_tokens = self.queued_tokens.len();

        // Apply the parser
        let parsed_token = closure(self)?;

        // Fail if our reader was not advanced
        if self.tokenizer.get_position() == position
            && self.queued_tokens.len() == n_buffered_tokens
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
    pub fn parse_any_number_of<T: Debug, F>(&mut self, closure: F) -> Vec<T>
    where
        F: Fn(&mut Self) -> Result<T, ParseError>,
    {
        let mut parsed_tokens = vec![];
        let mut state_before_end_token = self.clone();
        let mut iterations = 0;

        while let Ok(parsed_value) = closure(self) {
            if iterations == MAX_ITERATIONS {
                log::warn!("Exceeded maximum number of iterations, skipping...");
                break;
            }

            state_before_end_token = self.clone();
            parsed_tokens.push(parsed_value);

            iterations += 1;
        }

        // Reset to the last valid state to avoid accidentally consuming too many tokens
        *self = state_before_end_token;

        parsed_tokens
    }

    /// Apply a parser but don't throw an error if parsing fails.
    /// If no value could be parsed, `None` is returned and the internal
    /// state is not advanced.
    ///
    /// # Specification
    /// <https://w3c.github.io/csswg-drafts/css-values-4/#mult-opt>
    pub fn parse_optional_value<T, F>(&mut self, closure: F) -> Option<T>
    where
        F: Fn(&mut Self) -> Result<T, ParseError>,
    {
        let state = self.clone();
        let x = closure(self);
        match x {
            Ok(parsed_value) => Some(parsed_value),
            Err(_) => {
                *self = state;
                None
            },
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

    pub fn expect_percentage(&mut self) -> Result<Number, ParseError> {
        if let Some(Token::Percentage(percentage)) = self.next_token_ignoring_whitespace() {
            Ok(percentage)
        } else {
            Err(ParseError)
        }
    }

    pub fn expect_identifier(&mut self) -> Result<InternedString, ParseError> {
        if let Some(Token::Ident(identifier)) = self.next_token_ignoring_whitespace() {
            Ok(identifier)
        } else {
            Err(ParseError)
        }
    }

    pub fn expect_number(&mut self) -> Result<Number, ParseError> {
        if let Some(Token::Number(n)) = self.next_token_ignoring_whitespace() {
            Ok(n)
        } else {
            Err(ParseError)
        }
    }

    pub fn parse<T: CSSParse<'a>>(&mut self) -> Result<T, ParseError> {
        T::parse(self)
    }

    #[must_use]
    pub fn parse_optional<T: CSSParse<'a>>(&mut self) -> Option<T> {
        self.parse_optional_value(T::parse)
    }

    pub fn parse_four_sided_property<T: CSSParse<'a> + Copy>(
        &mut self,
    ) -> Result<Sides<T>, ParseError> {
        let first: T = self.parse()?;

        let Some(second) = self.parse_optional_value(T::parse) else {
            // If only one value is supplied, it is used for all four sides
            return Ok(Sides::all(first));
        };

        let Some(third) = self.parse_optional_value(T::parse) else {
            // If two values are supplied then the first one is used for the
            // top/bottom and the second one is used for left/right
            return Ok(Sides {
                top: first,
                right: second,
                bottom: first,
                left: second,
            });
        };

        let Some(fourth) = self.parse_optional_value(T::parse) else {
            // If three values are supplied then the first one is used for the
            // top, the second is used for left/right and the third is used for the bottom
            return Ok(Sides {
                top: first,
                right: second,
                bottom: third,
                left: second,
            });
        };

        Ok(Sides {
            top: first,
            right: second,
            bottom: third,
            left: fourth,
        })
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
        let parsed_value = Self::parse_complete(&mut parser)?;

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

        if parser.next_token_ignoring_whitespace().is_some() {
            return Err(ParseError);
        }
        Ok(parsed_value)
    }
}

impl<'a, T: CSSParse<'a> + Debug> CSSParse<'a> for Option<T> {
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        Ok(parser.parse_optional_value(T::parse))
    }
}
