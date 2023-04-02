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

use crate::{
    rule_parser::{ParsedRule, RuleParser},
    tokenizer::{Token, Tokenizer},
};
use bitflags::bitflags;
use std::fmt::Debug;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum MixedWithDeclarations {
    True,
    #[default]
    False,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TopLevel {
    True,
    #[default]
    False,
}

// /// <https://drafts.csswg.org/css-syntax/#css-decode-bytes>
// #[inline]
// pub fn decode_bytes(bytes: &[u8]) -> Cow<'_, str> {
//     // FIXME: make this spec compliant!
//     //        currently, we assume every stylesheet to be UTF-8 only
//     String::from_utf8_lossy(bytes)
// }

// /// <https://drafts.csswg.org/css-syntax/#parse-stylesheet>
// /// The stylesheet only borrows from the string source - you need to call `decode_bytes` first
// /// and keep a reference to the source around to use it.
// pub fn parse_stylesheet<'a, N: Normalize<'a>>(
//     input_not_normalized: N,
//     location: Option<URL>,
// ) -> Stylesheet<'a> {
//     // Normalize input, and set input to the result.
//     // See https://drafts.csswg.org/css-syntax/#normalize-into-a-token-stream
//     let input = input_not_normalized.normalize();
//     let mut parser = Parser::new(input);

//     // Create a new stylesheet, with its location set to location (or null, if location was not passed).
//     let mut stylesheet = Stylesheet::new(location);

//     // Consume a list of rules from input, with the top-level flag set, and set the stylesheet’s value to the result.
//     stylesheet.rules = parser.consume_list_of_rules(TopLevel::True);

//     // Return the stylesheet.
//     stylesheet
// }

// /// <https://drafts.csswg.org/css-syntax-3/#css-parse-a-comma-separated-list-according-to-a-css-grammar>
// pub fn parse_comma_seperated_list_according_to_css_grammar<'a, N: Normalize<'a>, G: CssGrammar>(
//     input: N,
// ) -> Vec<Result<G, G::ParseError>> {
//     // Normalize input, and set input to the result.
//     let input = input.normalize();

//     // If input contains only <whitespace-token>s, return an empty list.
//     if input
//         .iter()
//         .all(|value| matches!(value, ComponentValue::Token(Token::Whitespace)))
//     {
//         return vec![];
//     }

//     // Parse a comma-separated list of component values from input, and let list be the return value.
//     let component_value_lists = parse_comma_seperated_list_of_component_values(input);

//     // For each item of list, replace item with the result of parsing item with grammar.
//     let mut parsed_items = Vec::with_capacity(component_value_lists.len());
//     for component_values in component_value_lists {
//         parsed_items.push(G::parse(&component_values))
//     }

//     // Return list.
//     parsed_items
// }

// /// <https://drafts.csswg.org/css-syntax-3/#parse-a-list-of-component-values>
// pub fn parse_list_of_component_values<'a, N: Normalize<'a>>(input: N) -> Vec<ComponentValue<'a>> {
//     // Normalize input, and set input to the result.
//     let input = input.normalize();
//     let mut parser = Parser::new(input);

//     // Repeatedly consume a component value from input until an <EOF-token> is returned, appending the returned values (except the final <EOF-token>) into a list.
//     let mut values = vec![];
//     loop {
//         let component_value = parser.consume_component_value();

//         if let ComponentValue::EOF = component_value {
//             break;
//         }

//         values.push(component_value);
//     }

//     // Return the list.
//     values
// }

// /// <https://drafts.csswg.org/css-syntax-3/#parse-a-comma-separated-list-of-component-values>
// fn parse_comma_seperated_list_of_component_values<'a, N: Normalize<'a>>(
//     input: N,
// ) -> Vec<Vec<ComponentValue<'a>>> {
//     // Normalize input, and set input to the result.
//     let input = input.normalize();
//     let mut parser = Parser::new(input);

//     // Let list of cvls be an initially empty list of component value lists.
//     let mut component_value_lists = vec![];

//     let mut done = false;
//     while !done {
//         let mut component_values = vec![];
//         loop {
//             // Repeatedly consume a component value from input until an <EOF-token> or <comma-token> is returned,
//             // appending the returned values (except the final <EOF-token> or <comma-token>) into a list.
//             match parser.consume_component_value() {
//                 ComponentValue::EOF => {
//                     done = true;
//                     break;
//                 },
//                 ComponentValue::Token(Token::Comma) => break,
//                 other => component_values.push(other),
//             }
//         }

//         // Append the list to list of cvls.
//         component_value_lists.push(component_values);
//     }

//     // Return list of cvls.
//     component_value_lists
// }

#[derive(Clone, Debug)]
pub struct Parser<'a> {
    tokenizer: Tokenizer<'a>,
    buffered_token: Option<Token<'a>>,
    stop_at: Option<ParserDelimiter>,
    stopped: bool,
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct ParserDelimiter: u8 {
        const CURLY_BRACE_OPEN = 0b00000001;
        const CURLY_BRACE_CLOSE = 0b00000010;
        const SEMICOLON = 0b00000100;
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
                    if top_level == TopLevel::True {
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
        let prelude_ends_at = if mixed_with_declarations == MixedWithDeclarations::True {
            ParserDelimiter::CURLY_BRACE_OPEN | ParserDelimiter::SEMICOLON
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

    // /// <https://drafts.csswg.org/css-syntax/#consume-an-at-rule>
    // pub fn consume_at_rule(&mut self) -> Rule<'a> {
    //     // Consume the next input token.
    //     let _token = self.next_token();

    //     // Create a new at-rule with its name set to the value of the current input token, its prelude initially set to an empty list, and its value initially set to nothing.
    //     let name = String::new(); // FIXME whats the value of the current input token?
    //     let mut prelude = vec![];
    //     let mut block = None;

    //     // Repeatedly consume the next input token:
    //     loop {
    //         match self.next_token() {
    //             Some(ComponentValue::Token(Token::Semicolon)) => {
    //                 // Return the at-rule.
    //                 return Rule::AtRule(AtRule::new(name, prelude, block));
    //             },
    //             None => {
    //                 // This is a parse error.
    //                 log::warn!(target: "css", "Parse Error: EOF in @Rule");

    //                 // Return the at-rule.
    //                 return Rule::AtRule(AtRule::new(name, prelude, block));
    //             },
    //             Some(ComponentValue::Token(Token::CurlyBraceOpen)) => {
    //                 // Consume a simple block and assign it to the at-rule’s block.
    //                 block = Some(self.consume_simple_block(BlockDelimiter::CurlyBrace))

    //                 // Return the at-rule.
    //             },
    //             Some(ComponentValue::Block(
    //                 block @ SimpleBlock {
    //                     delimiter: BlockDelimiter::CurlyBrace,
    //                     ..
    //                 },
    //             )) => {
    //                 // Assign the block to the at-rule’s block. Return the at-rule.
    //                 return Rule::AtRule(AtRule::new(name, prelude, Some(block.clone())));
    //             },
    //             _ => {
    //                 // Reconsume the current input token
    //                 self.reconsume();

    //                 //  Consume a component value
    //                 let component_value = self.consume_component_value();

    //                 // Append the returned value to the at-rule’s prelude.
    //                 prelude.push(component_value);
    //             },
    //         }
    //     }
    // }

    // /// <https://drafts.csswg.org/css-syntax/#consume-a-simple-block>
    // pub fn consume_simple_block(&mut self, delimiter: BlockDelimiter) -> SimpleBlock<'a> {
    //     // The ending token is the mirror variant of the current input token.
    //     let end_token = delimiter.end_token();

    //     // Create a simple block with its associated token set to the current input token and with its value initially set to an empty list.
    //     let mut value = vec![];

    //     // Repeatedly consume the next input token and process it as follows:
    //     loop {
    //         let next_token = self.next_token();

    //         if next_token.is_none() {
    //             // This is a parse error.
    //             log::warn!(target: "css", "Parse Error: EOF in simple block");

    //             // Return the block.
    //             return SimpleBlock::new(delimiter, value);
    //         } else if let Some(ComponentValue::Token(token)) = next_token && token == &end_token {
    //             // Return the block.
    //             return SimpleBlock::new(delimiter, value);
    //         } else {
    //             // Reconsume the current input token
    //             self.reconsume();

    //             // Consume a component value and append it to the value of the block.
    //             value.push(self.consume_component_value())
    //         }
    //     }
    // }

    // /// <https://drafts.csswg.org/css-syntax/#consume-a-component-value>
    // pub fn consume_component_value(&mut self) -> ComponentValue<'a> {
    //     // Consume the next input token.
    //     match self.next_token() {
    //         // If the current input token is a <{-token>, <[-token>, or <(-token>, consume a simple block and return it.
    //         Some(ComponentValue::Token(Token::CurlyBraceOpen)) => {
    //             ComponentValue::Block(self.consume_simple_block(BlockDelimiter::CurlyBrace))
    //         },
    //         Some(ComponentValue::Token(Token::BracketOpen)) => {
    //             ComponentValue::Block(self.consume_simple_block(BlockDelimiter::Bracket))
    //         },
    //         Some(ComponentValue::Token(Token::ParenthesisOpen)) => {
    //             ComponentValue::Block(self.consume_simple_block(BlockDelimiter::Parenthesis))
    //         },
    //         Some(ComponentValue::Token(Token::Function(name))) => {
    //             let fn_name = name.clone().into_owned();
    //             ComponentValue::Function(self.consume_function(fn_name))
    //         },
    //         Some(other) => other.clone(),
    //         None => ComponentValue::EOF,
    //     }
    // }

    // /// <https://drafts.csswg.org/css-syntax/#consume-a-function>
    // pub fn consume_function(&mut self, name: String) -> Function<'a> {
    //     // Create a function with its name equal to the value of the current input token and with its value initially set to an empty list.
    //     let mut value = vec![];

    //     // Repeatedly consume the next input token and process it as follows:
    //     loop {
    //         match self.next_token() {
    //             Some(ComponentValue::Token(Token::ParenthesisClose)) => {
    //                 // Return the function.
    //                 return Function::new(name, value);
    //             },
    //             None => {
    //                 // This is a parse error.
    //                 log::warn!(target: "css", "Parse Error: EOF in function");

    //                 // Return the function.
    //                 return Function::new(name, value);
    //             },
    //             _ => {
    //                 // Reconsume the current input token
    //                 self.reconsume();

    //                 // Consume a component value and append the returned value to the function’s value.
    //                 value.push(self.consume_component_value());
    //             },
    //         }
    //     }
    // }

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
    /// # Specification
    /// <https://w3c.github.io/csswg-drafts/css-values-4/#mult-zero-plus>
    pub fn parse_any_number_of<T: Debug, F>(&mut self, closure: F) -> Vec<T>
    where
        F: Fn(&mut Self) -> Result<T, ParseError>,
    {
        let mut parsed_tokens = vec![];
        let mut state_before_end_token = self.state();
        while let Ok(parsed_value) = closure(self) {
            state_before_end_token = self.state();
            parsed_tokens.push(parsed_value);

            if self.expect_whitespace().is_err() {
                break;
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
