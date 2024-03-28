use sl_std::ring_buffer::RingBuffer;

use super::{Lexer, Punctuator, Token};
use crate::parser::SyntaxError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GoalSymbol {
    Script,
    Module,
}

const MAX_PEEK_AHEAD: usize = 3;

/// We set the buffer to be twice as large since there might be newlines inbetween
/// the tokens, which we have to account for.
/// Note that we collapse line terminator sequences into a single one, since they are meaningless.
const TOKEN_BUFFER_SIZE: usize = 2 * MAX_PEEK_AHEAD;

pub struct Tokenizer<'a, const BUFFER_SIZE: usize = TOKEN_BUFFER_SIZE> {
    buffered_tokens: RingBuffer<Token, BUFFER_SIZE>,
    lexer: Lexer<'a>,
    strict: bool,
    goal_symbol: GoalSymbol,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkipLineTerminators {
    Yes,
    No,
}

impl<'a, const BUFFER_SIZE: usize> Tokenizer<'a, BUFFER_SIZE> {
    #[must_use]
    pub fn new(source_text: &'a str, goal_symbol: GoalSymbol) -> Self {
        Self {
            buffered_tokens: RingBuffer::default(),
            lexer: Lexer::new(source_text),
            strict: false,
            goal_symbol,
        }
    }

    #[must_use]
    pub const fn is_strict(&self) -> bool {
        self.strict
    }

    #[must_use]
    pub const fn goal_symbol(&self) -> GoalSymbol {
        self.goal_symbol
    }

    pub fn set_strict(&mut self, strict: bool) {
        self.strict = strict;
    }

    pub fn expect_keyword(&mut self, keyword: &str) -> Result<(), SyntaxError> {
        match self.next(SkipLineTerminators::Yes)? {
            Some(Token::Identifier(ident)) if ident == keyword => Ok(()),
            _ => Err(self.syntax_error()),
        }
    }

    pub fn expect_punctuator(&mut self, punctuator: Punctuator) -> Result<(), SyntaxError> {
        match self.next(SkipLineTerminators::Yes)? {
            Some(Token::Punctuator(p)) if p == punctuator => Ok(()),
            _ => Err(self.syntax_error()),
        }
    }

    pub fn expect_no_line_terminator(&mut self) -> Result<(), SyntaxError> {
        let is_line_terminator = matches!(
            self.peek(0, SkipLineTerminators::No),
            Ok(Some(Token::LineTerminator))
        );

        if is_line_terminator {
            Err(self.syntax_error())
        } else {
            Ok(())
        }
    }

    /// Make sure there is at least one more token in the buffer which is not a newline
    fn tokenize_next(&mut self) -> Result<(), SyntaxError> {
        if self.buffered_tokens.is_full() {
            return Ok(());
        }

        let mut last_is_newline = false;
        while let Some(token) = self.lexer.next_token()? {
            if token.is_line_terminator() {
                if !last_is_newline {
                    last_is_newline = true;
                    self.buffered_tokens.push(token);
                }
            } else {
                self.buffered_tokens.push(token);
                break;
            }
        }

        Ok(())
    }

    pub fn syntax_error(&self) -> SyntaxError {
        self.lexer.syntax_error()
    }

    pub fn advance(&mut self, n: usize) {
        for _ in 0..n {
            _ = self.buffered_tokens.pop_front();
        }
    }

    /// Peek `n` tokens ahead of the current one
    pub fn peek(
        &mut self,
        n: usize,
        skip_line_terminators: SkipLineTerminators,
    ) -> Result<Option<&Token>, SyntaxError> {
        debug_assert!(n < BUFFER_SIZE);

        let mut i = 0;

        while i != n + 1 {
            let future_token = self.buffered_tokens.peek_front(i);
            match future_token {
                Some(Token::LineTerminator)
                    if skip_line_terminators == SkipLineTerminators::Yes =>
                {
                    continue;
                },
                Some(_) => {
                    i += 1;
                },
                None => {
                    // We have reached the end of the tokens we have peeked so far, from now on we need to peek more
                    while i != n + 1 {
                        match self.lexer.next_token()? {
                            Some(Token::LineTerminator)
                                if skip_line_terminators == SkipLineTerminators::Yes =>
                            {
                                self.buffered_tokens.push(Token::LineTerminator);
                                continue;
                            },
                            Some(other_token) => {
                                self.buffered_tokens.push(other_token);
                                i += 1;
                            },
                            None => {
                                // End of input
                                return Ok(None);
                            },
                        }
                    }
                    break;
                },
            }
        }

        let peeked_token = self.buffered_tokens.peek_front(n);
        Ok(peeked_token)
    }

    pub fn next(
        &mut self,
        skip_line_terminators: SkipLineTerminators,
    ) -> Result<Option<Token>, SyntaxError> {
        if self.buffered_tokens.is_empty() {
            self.tokenize_next()?;
        }

        let next_token = if let Some(token) = self.buffered_tokens.pop_front() {
            if token.is_line_terminator() && skip_line_terminators == SkipLineTerminators::Yes {
                // Next token is known to not be a line terminator
                self.buffered_tokens.pop_front()
            } else {
                Some(token)
            }
        } else {
            // There are no more tokens, we're done
            None
        };

        Ok(next_token)
    }
}
