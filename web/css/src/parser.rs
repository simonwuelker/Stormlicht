use std::borrow::Cow;

use crate::{
    tokenizer::{Token, Tokenizer},
    tree::{AtRule, ComponentValue, Rule, SimpleBlock},
};

#[derive(Clone, Debug)]
pub struct Parser<'a> {
    tokenizer: Tokenizer<'a>,
    token_to_reconsume: Option<Token<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            tokenizer: Tokenizer::new(source),
            token_to_reconsume: None,
        }
    }

    fn next_token(&mut self) -> Token<'a> {
        self.token_to_reconsume
            .take()
            .unwrap_or(self.tokenizer.next_token())
    }

    /// https://drafts.csswg.org/css-syntax/#reconsume-the-current-input-token
    fn reconsume(&mut self, token: Token<'a>) {
        debug_assert!(self.token_to_reconsume.is_none());

        self.token_to_reconsume = Some(token);
    }

    /// https://drafts.csswg.org/css-syntax/#consume-list-of-rules
    pub fn consume_list_of_rules(&mut self, top_level: bool) -> Vec<Rule> {
        // Create an initially empty list of rules.
        let mut rules = vec![];

        // Repeatedly consume the next input token:
        loop {
            match self.tokenizer.next_token() {
                Token::Whitespace => {
                    // Do nothing.
                },
                Token::EOF => {
                    // Return the list of rules.
                    return rules;
                },
                token @ (Token::CommentDeclarationOpen | Token::CommentDeclarationClose) => {
                    // If the top-level flag is set,
                    if top_level {
                        // do nothing.
                    }
                    // Otherwise,
                    else {
                        //  reconsume the current input token
                        self.reconsume(token);

                        // Consume a qualified rule
                        // If anything is returned, append it to the list of rules.
                        rules.extend(self.consume_qualified_rule())
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
                    rules.extend(self.consume_qualified_rule());
                },
            }
        }
    }

    /// https://drafts.csswg.org/css-syntax/#consume-a-qualified-rule
    pub fn consume_qualified_rule(&mut self) -> Vec<Rule<'a>> {
        todo!()
    }

    /// https://drafts.csswg.org/css-syntax/#consume-an-at-rule
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
                Token::Semicolon => {
                    // Return the at-rule.
                    return Rule::AtRule(AtRule::new(Cow::Owned(name), prelude, block));
                },
                Token::EOF => {
                    // This is a parse error.
                    // Return the at-rule.
                    return Rule::AtRule(AtRule::new(Cow::Owned(name), prelude, block));
                },
                Token::CurlyBraceOpen => {
                    // Consume a simple block and assign it to the at-rule’s block.
                    block = Some(self.consume_simple_block())

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
    pub fn consume_simple_block(&mut self) -> SimpleBlock<'a> {
        todo!()
    }

    /// https://drafts.csswg.org/css-syntax/#consume-a-component-value
    pub fn consume_component_value(&mut self) -> ComponentValue<'a> {
        todo!()
    }
}
