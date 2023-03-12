use std::borrow::Cow;

use url::URL;

use crate::{
    selectors::CssGrammar,
    stylesheet::Stylesheet,
    tokenizer::{Token, Tokenizer},
    tree::{AtRule, BlockDelimiter, ComponentValue, Function, QualifiedRule, Rule, SimpleBlock},
};

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

#[derive(Clone, Debug)]
pub struct Parser<'a> {
    tokens: Vec<ComponentValue<'a>>,
    current_token_ptr: usize,
}

/// <https://drafts.csswg.org/css-syntax/#css-decode-bytes>
#[inline]
pub fn decode_bytes(bytes: &[u8]) -> Cow<'_, str> {
    // FIXME: make this spec compliant!
    //        currently, we assume every stylesheet to be UTF-8 only
    String::from_utf8_lossy(bytes)
}

/// <https://drafts.csswg.org/css-syntax/#parse-stylesheet>
/// The stylesheet only borrows from the string source - you need to call `decode_bytes` first
/// and keep a reference to the source around to use it.
pub fn parse_stylesheet<'a, N: Normalize<'a>>(
    input_not_normalized: N,
    location: Option<URL>,
) -> Stylesheet<'a> {
    // Normalize input, and set input to the result.
    // See https://drafts.csswg.org/css-syntax/#normalize-into-a-token-stream
    let input = input_not_normalized.normalize();
    let mut parser = Parser::new(input);

    // Create a new stylesheet, with its location set to location (or null, if location was not passed).
    let mut stylesheet = Stylesheet::new(location);

    // Consume a list of rules from input, with the top-level flag set, and set the stylesheet’s value to the result.
    stylesheet.rules = parser.consume_list_of_rules(TopLevel::True);

    // Return the stylesheet.
    stylesheet
}

/// <https://drafts.csswg.org/css-syntax-3/#css-parse-a-comma-separated-list-according-to-a-css-grammar>
pub fn parse_comma_seperated_list_according_to_css_grammar<'a, N: Normalize<'a>, G: CssGrammar>(
    input: N,
) -> Vec<Result<G, G::ParseError>> {
    // Normalize input, and set input to the result.
    let input = input.normalize();

    // If input contains only <whitespace-token>s, return an empty list.
    if input
        .iter()
        .all(|value| matches!(value, ComponentValue::Token(Token::Whitespace)))
    {
        return vec![];
    }

    // Parse a comma-separated list of component values from input, and let list be the return value.
    let component_value_lists = parse_comma_seperated_list_of_component_values(input);

    // For each item of list, replace item with the result of parsing item with grammar.
    let mut parsed_items = Vec::with_capacity(component_value_lists.len());
    for component_values in component_value_lists {
        parsed_items.push(G::parse(&component_values))
    }

    // Return list.
    parsed_items
}

/// <https://drafts.csswg.org/css-syntax-3/#parse-a-list-of-component-values>
pub fn parse_list_of_component_values<'a, N: Normalize<'a>>(input: N) -> Vec<ComponentValue<'a>> {
    // Normalize input, and set input to the result.
    let input = input.normalize();
    let mut parser = Parser::new(input);

    // Repeatedly consume a component value from input until an <EOF-token> is returned, appending the returned values (except the final <EOF-token>) into a list.
    let mut values = vec![];
    loop {
        let component_value = parser.consume_component_value();

        if let ComponentValue::EOF = component_value {
            break;
        }

        values.push(component_value);
    }

    // Return the list.
    values
}

/// <https://drafts.csswg.org/css-syntax-3/#parse-a-comma-separated-list-of-component-values>
fn parse_comma_seperated_list_of_component_values<'a, N: Normalize<'a>>(
    input: N,
) -> Vec<Vec<ComponentValue<'a>>> {
    // Normalize input, and set input to the result.
    let input = input.normalize();
    let mut parser = Parser::new(input);

    // Let list of cvls be an initially empty list of component value lists.
    let mut component_value_lists = vec![];

    let mut done = false;
    while !done {
        let mut component_values = vec![];
        loop {
            // Repeatedly consume a component value from input until an <EOF-token> or <comma-token> is returned,
            // appending the returned values (except the final <EOF-token> or <comma-token>) into a list.
            match parser.consume_component_value() {
                ComponentValue::EOF => {
                    done = true;
                    break;
                },
                ComponentValue::Token(Token::Comma) => break,
                other => component_values.push(other),
            }
        }

        // Append the list to list of cvls.
        component_value_lists.push(component_values);
    }

    // Return list of cvls.
    component_value_lists
}

impl<'a> Parser<'a> {
    fn new(tokens: Vec<ComponentValue<'a>>) -> Self {
        Self {
            tokens: tokens,
            current_token_ptr: 0,
        }
    }

    fn next_token(&mut self) -> Option<&ComponentValue<'a>> {
        if self.current_token_ptr < self.tokens.len() {
            let token = &self.tokens[self.current_token_ptr];
            self.current_token_ptr += 1;
            Some(token)
        } else {
            None
        }
    }

    /// <https://drafts.csswg.org/css-syntax/#reconsume-the-current-input-token>
    /// The reason this takes an `Option<Token>` instead of just a `Token` is
    /// that a lot of algorithms reconsume `EOF` (`None`) tokens
    fn reconsume(&mut self) {
        self.current_token_ptr -= 1;
    }

    /// <https://drafts.csswg.org/css-syntax/#consume-list-of-rules>
    pub fn consume_list_of_rules(&mut self, top_level: TopLevel) -> Vec<Rule<'a>> {
        // Create an initially empty list of rules.
        let mut rules = vec![];

        // Repeatedly consume the next input token:
        loop {
            match self.next_token() {
                Some(ComponentValue::Token(Token::Whitespace)) => {
                    // Do nothing.
                },
                None => {
                    // Return the list of rules.
                    return rules;
                },
                Some(ComponentValue::Token(
                    Token::CommentDeclarationOpen | Token::CommentDeclarationClose,
                )) => {
                    // If the top-level flag is set,
                    if top_level == TopLevel::True {
                        // do nothing.
                    }
                    // Otherwise,
                    else {
                        //  reconsume the current input token
                        self.reconsume();

                        // Consume a qualified rule
                        // If anything is returned, append it to the list of rules.
                        if let Some(rule) =
                            self.consume_qualified_rule(MixedWithDeclarations::default())
                        {
                            rules.push(rule);
                        }
                    }
                },
                Some(ComponentValue::Token(Token::AtKeyword(_))) => {
                    // Reconsume the current input token
                    self.reconsume();

                    // Consume an at-rule, and append the returned value to the list of rules.
                    rules.push(self.consume_at_rule());
                },
                _ => {
                    // Reconsume the current input token
                    self.reconsume();

                    // Consume a qualified rule. If anything is returned, append it to the list of rules.
                    if let Some(rule) =
                        self.consume_qualified_rule(MixedWithDeclarations::default())
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
        mixed_with_declarations: MixedWithDeclarations,
    ) -> Option<Rule<'a>> {
        // Create a new qualified rule with its prelude initially set to an empty list, and its block initially set to nothing.

        // NOTE: a block can't be "nothing" but since we never return this "nothing" we simply don't care about it
        // until we get a value

        let mut prelude = vec![];

        // Repeatedly consume the next input token:
        loop {
            match self.next_token() {
                None => {
                    // This is a parse error.
                    log::warn!(target: "css", "Parse Error: EOF in qualified rule");

                    // Return nothing.
                    return None;
                },
                Some(ComponentValue::Token(Token::Semicolon)) => {
                    // If mixed with declarations is true
                    if mixed_with_declarations == MixedWithDeclarations::True {
                        // this is a parse error;
                        log::warn!(target: "css", "Parse Error: Semicolon in qualified rule with mixed_with_declarations=true");

                        // return nothing.
                        return None;
                    }
                    // Otherwise,
                    else {
                        // append a <semicolon-token> to the qualified rule’s prelude.
                        prelude.push(ComponentValue::Token(Token::Semicolon));
                    }
                },
                Some(ComponentValue::Token(Token::CurlyBraceOpen)) => {
                    // Consume a simple block and assign it to the qualified rule’s block
                    let block = self.consume_simple_block(BlockDelimiter::CurlyBrace);

                    // Return the qualified rule.
                    return Some(Rule::QualifiedRule(QualifiedRule::new(prelude, block)));
                },
                Some(ComponentValue::Block(
                    block @ SimpleBlock {
                        delimiter: BlockDelimiter::CurlyBrace,
                        ..
                    },
                )) => {
                    // Assign the block to the qualified rule’s block. Return the qualified rule.
                    return Some(Rule::QualifiedRule(QualifiedRule::new(
                        prelude,
                        block.clone(),
                    )));
                },
                _ => {
                    // Reconsume the current input token
                    self.reconsume();

                    // Consume a component value
                    let component_value = self.consume_component_value();

                    // Append the returned value to the qualified rule’s prelude.
                    prelude.push(component_value);
                },
            }
        }
    }

    /// <https://drafts.csswg.org/css-syntax/#consume-an-at-rule>
    pub fn consume_at_rule(&mut self) -> Rule<'a> {
        // Consume the next input token.
        let _token = self.next_token();

        // Create a new at-rule with its name set to the value of the current input token, its prelude initially set to an empty list, and its value initially set to nothing.
        let name = String::new(); // FIXME whats the value of the current input token?
        let mut prelude = vec![];
        let mut block = None;

        // Repeatedly consume the next input token:
        loop {
            match self.next_token() {
                Some(ComponentValue::Token(Token::Semicolon)) => {
                    // Return the at-rule.
                    return Rule::AtRule(AtRule::new(name, prelude, block));
                },
                None => {
                    // This is a parse error.
                    log::warn!(target: "css", "Parse Error: EOF in @Rule");

                    // Return the at-rule.
                    return Rule::AtRule(AtRule::new(name, prelude, block));
                },
                Some(ComponentValue::Token(Token::CurlyBraceOpen)) => {
                    // Consume a simple block and assign it to the at-rule’s block.
                    block = Some(self.consume_simple_block(BlockDelimiter::CurlyBrace))

                    // Return the at-rule.
                },
                Some(ComponentValue::Block(
                    block @ SimpleBlock {
                        delimiter: BlockDelimiter::CurlyBrace,
                        ..
                    },
                )) => {
                    // Assign the block to the at-rule’s block. Return the at-rule.
                    return Rule::AtRule(AtRule::new(name, prelude, Some(block.clone())));
                },
                _ => {
                    // Reconsume the current input token
                    self.reconsume();

                    //  Consume a component value
                    let component_value = self.consume_component_value();

                    // Append the returned value to the at-rule’s prelude.
                    prelude.push(component_value);
                },
            }
        }
    }

    /// <https://drafts.csswg.org/css-syntax/#consume-a-simple-block>
    pub fn consume_simple_block(&mut self, delimiter: BlockDelimiter) -> SimpleBlock<'a> {
        // The ending token is the mirror variant of the current input token.
        let end_token = delimiter.end_token();

        // Create a simple block with its associated token set to the current input token and with its value initially set to an empty list.
        let mut value = vec![];

        // Repeatedly consume the next input token and process it as follows:
        loop {
            let next_token = self.next_token();

            if next_token.is_none() {
                // This is a parse error.
                log::warn!(target: "css", "Parse Error: EOF in simple block");

                // Return the block.
                return SimpleBlock::new(delimiter, value);
            } else if let Some(ComponentValue::Token(token)) = next_token && token == &end_token {
                // Return the block.
                return SimpleBlock::new(delimiter, value);
            } else {
                // Reconsume the current input token
                self.reconsume();

                // Consume a component value and append it to the value of the block.
                value.push(self.consume_component_value())
            }
        }
    }

    /// <https://drafts.csswg.org/css-syntax/#consume-a-component-value>
    pub fn consume_component_value(&mut self) -> ComponentValue<'a> {
        // Consume the next input token.
        match self.next_token() {
            // If the current input token is a <{-token>, <[-token>, or <(-token>, consume a simple block and return it.
            Some(ComponentValue::Token(Token::CurlyBraceOpen)) => {
                ComponentValue::Block(self.consume_simple_block(BlockDelimiter::CurlyBrace))
            },
            Some(ComponentValue::Token(Token::BracketOpen)) => {
                ComponentValue::Block(self.consume_simple_block(BlockDelimiter::Bracket))
            },
            Some(ComponentValue::Token(Token::ParenthesisOpen)) => {
                ComponentValue::Block(self.consume_simple_block(BlockDelimiter::Parenthesis))
            },
            Some(ComponentValue::Token(Token::Function(name))) => {
                let fn_name = name.clone().into_owned();
                ComponentValue::Function(self.consume_function(fn_name))
            },
            Some(other) => other.clone(),
            None => ComponentValue::EOF,
        }
    }

    /// <https://drafts.csswg.org/css-syntax/#consume-a-function>
    pub fn consume_function(&mut self, name: String) -> Function<'a> {
        // Create a function with its name equal to the value of the current input token and with its value initially set to an empty list.
        let mut value = vec![];

        // Repeatedly consume the next input token and process it as follows:
        loop {
            match self.next_token() {
                Some(ComponentValue::Token(Token::ParenthesisClose)) => {
                    // Return the function.
                    return Function::new(name, value);
                },
                None => {
                    // This is a parse error.
                    log::warn!(target: "css", "Parse Error: EOF in function");

                    // Return the function.
                    return Function::new(name, value);
                },
                _ => {
                    // Reconsume the current input token
                    self.reconsume();

                    // Consume a component value and append the returned value to the function’s value.
                    value.push(self.consume_component_value());
                },
            }
        }
    }
}

pub trait Normalize<'a> {
    fn normalize(self) -> Vec<ComponentValue<'a>>;
}

impl<'a> Normalize<'a> for Tokenizer<'a> {
    fn normalize(self) -> Vec<ComponentValue<'a>> {
        self.into_iter().map(ComponentValue::Token).collect()
    }
}

impl<'a> Normalize<'a> for Vec<ComponentValue<'a>> {
    fn normalize(self) -> Vec<ComponentValue<'a>> {
        self
    }
}

impl<'a> Normalize<'a> for &'a str {
    fn normalize(self) -> Vec<ComponentValue<'a>> {
        Tokenizer::new(self).normalize()
    }
}