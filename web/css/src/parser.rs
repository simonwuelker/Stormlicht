use std::borrow::Cow;

use url::URL;

use crate::{
    stylesheet::Stylesheet,
    tokenizer::{Token, Tokenizer},
    tree::{
        AtRule, BlockDelimiter, ComponentValue, Function, PreservedToken, QualifiedRule, Rule,
        SimpleBlock,
    },
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
    tokenizer: Tokenizer<'a>,
    token_to_reconsume: Option<Token<'a>>,
}

/// https://drafts.csswg.org/css-syntax/#css-decode-bytes
#[inline]
fn decode_bytes(bytes: &[u8]) -> Cow<'_, str> {
    // FIXME: make this spec compliant!
    //        currently, we assume every stylesheet to be UTF-8 only
    String::from_utf8_lossy(bytes)
}

/// https://drafts.csswg.org/css-syntax/#parse-stylesheet
pub fn parse_stylesheet(bytes: &[u8], location: Option<URL>) -> Stylesheet {
    // If input is a byte stream for stylesheet, decode bytes from input, and set input to the result.
    let input = decode_bytes(bytes);

    // If input is a byte stream for stylesheet, decode bytes from input, and set input to the result.
    // See https://drafts.csswg.org/css-syntax/#normalize-into-a-token-stream
    let tokenizer = Tokenizer::new(&input);

    // Create a new stylesheet, with its location set to location (or null, if location was not passed).
    let mut stylesheet = Stylesheet::new(location);

    // Consume a list of rules from input, with the top-level flag set, and set the stylesheet’s value to the result.
    stylesheet.rules = Parser::new(tokenizer).consume_list_of_rules(TopLevel::True);

    // Return the stylesheet.
    stylesheet
}

impl<'a> Parser<'a> {
    fn new(source: Tokenizer<'a>) -> Self {
        Self {
            tokenizer: source,
            token_to_reconsume: None,
        }
    }

    fn next_token(&mut self) -> Token<'a> {
        match self.token_to_reconsume.take() {
            Some(token) => token,
            None => self.tokenizer.next_token(),
        }
    }

    /// https://drafts.csswg.org/css-syntax/#reconsume-the-current-input-token
    fn reconsume(&mut self, token: Token<'a>) {
        debug_assert!(self.token_to_reconsume.is_none());
        self.token_to_reconsume = Some(token);
    }

    /// https://drafts.csswg.org/css-syntax/#consume-list-of-rules
    pub fn consume_list_of_rules(&mut self, top_level: TopLevel) -> Vec<Rule> {
        // Create an initially empty list of rules.
        let mut rules = vec![];

        // Repeatedly consume the next input token:
        loop {
            match self.next_token() {
                Token::Whitespace => {
                    // Do nothing.
                },
                Token::EOF => {
                    // Return the list of rules.
                    return rules;
                },
                token @ (Token::CommentDeclarationOpen | Token::CommentDeclarationClose) => {
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
                        if let Some(rule) =
                            self.consume_qualified_rule(MixedWithDeclarations::default())
                        {
                            rules.push(rule);
                        }
                    }
                },
                token @ Token::AtKeyword(_) => {
                    // Reconsume the current input token
                    self.reconsume(token);

                    // Consume an at-rule, and append the returned value to the list of rules.
                    rules.push(self.consume_at_rule());
                },
                other => {
                    // Reconsume the current input token
                    self.reconsume(other);

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

    /// https://drafts.csswg.org/css-syntax/#consume-a-qualified-rule
    pub fn consume_qualified_rule(
        &mut self,
        mixed_with_declarations: MixedWithDeclarations,
    ) -> Option<Rule> {
        // Create a new qualified rule with its prelude initially set to an empty list, and its block initially set to nothing.

        // NOTE: a block can't be "nothing" but since we never return this "nothing" we simply don't care about it
        // until we get a value

        let mut prelude = vec![];

        // Repeatedly consume the next input token:
        loop {
            match self.next_token() {
                Token::EOF => {
                    // This is a parse error.
                    // Return nothing.
                    return None;
                },
                Token::Semicolon => {
                    // If mixed with declarations is true
                    if mixed_with_declarations == MixedWithDeclarations::True {
                        // this is a parse error;
                        // return nothing.
                        return None;
                    }
                    // Otherwise,
                    else {
                        // append a <semicolon-token> to the qualified rule’s prelude.
                        prelude.push(ComponentValue::Token(PreservedToken::Semicolon));
                    }
                },
                Token::CurlyBraceOpen => {
                    // Consume a simple block and assign it to the qualified rule’s block
                    let block = self.consume_simple_block(BlockDelimiter::CurlyBrace);

                    // Return the qualified rule.
                    return Some(Rule::QualifiedRule(QualifiedRule::new(prelude, block)));
                },
                other => {
                    // Reconsume the current input token
                    self.reconsume(other);

                    // Consume a component value
                    let component_value = self.consume_component_value();

                    // Append the returned value to the qualified rule’s prelude.
                    prelude.push(component_value);
                },
            }
        }
    }

    /// https://drafts.csswg.org/css-syntax/#consume-an-at-rule
    pub fn consume_at_rule(&mut self) -> Rule {
        // Consume the next input token.
        let _token = self.next_token();

        // Create a new at-rule with its name set to the value of the current input token, its prelude initially set to an empty list, and its value initially set to nothing.
        let name = String::new(); // FIXME whats the value of the current input token?
        let mut prelude = vec![];
        let mut block = None;

        // Repeatedly consume the next input token:
        loop {
            match self.next_token() {
                Token::Semicolon => {
                    // Return the at-rule.
                    return Rule::AtRule(AtRule::new(name, prelude, block));
                },
                Token::EOF => {
                    // This is a parse error.
                    // Return the at-rule.
                    return Rule::AtRule(AtRule::new(name, prelude, block));
                },
                Token::CurlyBraceOpen => {
                    // Consume a simple block and assign it to the at-rule’s block.
                    block = Some(self.consume_simple_block(BlockDelimiter::CurlyBrace))

                    // Return the at-rule.
                },
                // FIXME there's a rule which i dont understand here
                other => {
                    // Reconsume the current input token
                    self.reconsume(other);

                    //  Consume a component value
                    let component_value = self.consume_component_value();

                    // Append the returned value to the at-rule’s prelude.
                    prelude.push(component_value);
                },
            }
        }
    }

    /// https://drafts.csswg.org/css-syntax/#consume-a-simple-block
    #[allow(clippy::if_same_then_else)]
    pub fn consume_simple_block(&mut self, delimiter: BlockDelimiter) -> SimpleBlock {
        // The ending token is the mirror variant of the current input token.
        let end_token = delimiter.end_token();

        // Create a simple block with its associated token set to the current input token and with its value initially set to an empty list.
        let mut value = vec![];

        // Repeatedly consume the next input token and process it as follows:
        loop {
            let next_token = self.next_token();

            if next_token == end_token {
                // Return the block.
                return SimpleBlock::new(delimiter, value);
            } else if next_token == Token::EOF {
                // This is a parse error.
                // Return the block.
                return SimpleBlock::new(delimiter, value);
            } else {
                // Reconsume the current input token
                self.reconsume(next_token);

                // Consume a component value and append it to the value of the block.
                value.push(self.consume_component_value())
            }
        }
    }

    /// https://drafts.csswg.org/css-syntax/#consume-a-component-value
    pub fn consume_component_value(&mut self) -> ComponentValue {
        // Consume the next input token.
        match self.next_token() {
            // If the current input token is a <{-token>, <[-token>, or <(-token>, consume a simple block and return it.
            Token::CurlyBraceOpen => {
                ComponentValue::Block(self.consume_simple_block(BlockDelimiter::CurlyBrace))
            },
            Token::BracketOpen => {
                ComponentValue::Block(self.consume_simple_block(BlockDelimiter::Bracket))
            },
            Token::ParenthesisOpen => {
                ComponentValue::Block(self.consume_simple_block(BlockDelimiter::Parenthesis))
            },
            Token::Function(name) => {
                ComponentValue::Function(self.consume_function(name.into_owned()))
            },
            other => ComponentValue::Token(PreservedToken::from_regular_token(other)),
        }
    }

    /// https://drafts.csswg.org/css-syntax/#consume-a-function
    pub fn consume_function(&mut self, name: String) -> Function {
        // Create a function with its name equal to the value of the current input token and with its value initially set to an empty list.
        let mut value = vec![];

        // Repeatedly consume the next input token and process it as follows:
        loop {
            match self.next_token() {
                Token::ParenthesisClose => {
                    // Return the function.
                    return Function::new(name, value);
                },
                Token::EOF => {
                    // This is a parse error.
                    // Return the function.
                    return Function::new(name, value);
                },
                other => {
                    // Reconsume the current input token
                    self.reconsume(other);

                    // Consume a component value and append the returned value to the function’s value.
                    value.push(self.consume_component_value());
                },
            }
        }
    }
}
