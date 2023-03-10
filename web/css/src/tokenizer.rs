use std::borrow::Cow;

// characters are sort of unreadable and should
// be referenced via their name instead
const NEWLINE: char = '\n';
const TAB: char = '\t';
const WHITESPACE: char = ' ';
const APOSTROPHE: char = '\'';
const BACKSLASH: char = '\\';
const REPLACEMENT: char = '\u{FFFD}';

#[derive(Clone, Debug, PartialEq)]
pub enum HashFlag {
    Unrestricted,
    Id,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Token<'a> {
    Ident(Cow<'a, str>),
    AtKeyword(Cow<'a, str>),
    String(Cow<'a, str>),
    BadString(Cow<'a, str>),
    BadURI(Cow<'a, str>),
    BadComment(Cow<'a, str>),
    Hash(Cow<'a, str>, HashFlag),
    Number(Cow<'a, str>),
    Percentage(Cow<'a, str>),
    Dimension(Cow<'a, str>, Cow<'a, str>),
    URI(Cow<'a, str>),
    CommentDeclarationOpen,
    CommentDeclarationClose,
    Colon,
    Semicolon,
    CurlyBraceOpen,
    CurlyBraceClose,
    ParenthesisOpen,
    ParenthesisClose,
    BracketOpen,
    BracketClose,
    Whitespace,
    Comment,
    Function(Cow<'a, str>),
    Includes,
    Dashmatch,
    Comma,
    EOF,
    Delim(char),
}

#[derive(Clone, Copy, Debug)]
pub struct Tokenizer<'a> {
    source: &'a str,
    position: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            position: 0,
        }
    }

    fn reconsume(&mut self) {
        self.position -= 1;
    }

    fn peek_codepoint(&self, n: usize) -> Option<char> {
        self.source.chars().nth(self.position + n)
    }

    /// https://drafts.csswg.org/css-syntax/#check-if-two-code-points-are-a-valid-escape
    fn is_valid_escape(&self) -> bool {
        // If the first code point is not U+005C REVERSE SOLIDUS (\)
        if self.peek_codepoint(0) != Some(BACKSLASH) {
            // return false.
            false
        } else {
            // Otherwise, if the second code point is a newline return false.
            // Otherwise return true.
            self.peek_codepoint(1) != Some(NEWLINE)
        }
    }

    /// https://drafts.csswg.org/css-syntax/#check-if-three-code-points-would-start-an-ident-sequence
    fn is_valid_ident_start(&self) -> bool {
        todo!()
    }

    /// https://drafts.csswg.org/css-syntax/#check-if-three-code-points-would-start-a-number
    fn is_valid_number_start(&self) -> bool {
        todo!()
    }

    fn next_codepoint(&mut self) -> Option<char> {
        let c = self.source.chars().nth(self.position);
        self.advance(1);
        c
    }

    fn consume_whitespace(&mut self) {
        while matches!(self.peek_codepoint(0), Some(NEWLINE | TAB | WHITESPACE)) {
            self.advance(1);
        }
    }

    /// https://drafts.csswg.org/css-syntax/#consume-an-ident-sequence
    fn consume_ident_sequence(&mut self) -> String {
        // Let result initially be an empty string.
        let mut result = String::new();

        // Repeatedly consume the next input code point from the stream:
        loop {
            let n1 = self.next_codepoint();

            match n1 {
                Some(c) => {
                    // ident code point:
                    if is_ident_code_point(c) {
                        // Append the code point to result.
                        result.push(c);
                    }
                    // the stream starts with a valid escape:
                    else if self.is_valid_escape() {
                        // Consume an escaped code point.
                        let c = self.consume_escaped_codepoint();

                        // Append the returned code point to result.
                        result.push(c);
                    }
                    // anything else
                    else {
                        // Reconsume the current input code point.
                        self.reconsume();

                        // Return result.
                        return result;
                    }
                },
                None => {
                    // Reconsume the current input code point.
                    self.reconsume();

                    // Return result.
                    return result;
                },
            }
        }
    }

    fn consume_escaped_codepoint(&mut self) -> char {
        // Consume the next input code point.
        match self.next_codepoint() {
            Some(c @ ('0'..='9' | 'a'..='f' | 'A'..='F')) => {
                let mut num = c.to_digit(16).unwrap();

                // Consume as many hex digits as possible, but no more than 5.
                // Note that this means 1-6 hex digits have been consumed in total
                let mut digits_consumed = 0;
                while let Some(c @ ('0'..='9' | 'a'..='f' | 'A'..='F')) = self.next_codepoint() {
                    digits_consumed += 1;
                    num *= 16;
                    num += c.to_digit(16).unwrap();

                    if digits_consumed == 5 {
                        break;
                    }
                }

                // If this number is zero, or is for a surrogate, or is greater than the maximum allowed code point
                if matches!(num, 0 | 0xD800..=0xDFFF | 11000..) {
                    // return U+FFFD REPLACEMENT CHARACTER (�)
                    REPLACEMENT
                }
                // Otherwise,
                else {
                    // Otherwise, return the code point with that value.
                    char::from_u32(num).unwrap()
                }
            },
            None => {
                // This is a parse error.
                // Return U+FFFD REPLACEMENT CHARACTER (�).
                REPLACEMENT
            },
            Some(c) => {
                // Return the current input code point.
                c
            },
        }
    }

    /// https://drafts.csswg.org/css-syntax/#consume-a-numeric-token
    fn consume_numeric_token(&mut self) -> Token<'a> {
        todo!()
    }

    /// https://drafts.csswg.org/css-syntax/#consume-a-url-token
    fn consume_url_token(&mut self) -> Token<'a> {
        todo!()
    }

    /// https://drafts.csswg.org/css-syntax/#consume-an-ident-like-token
    fn consume_ident_like_token(&mut self) -> Token<'a> {
        // Consume an ident sequence, and let string be the result.
        let string = self.consume_ident_sequence();

        // If string’s value is an ASCII case-insensitive match for "url", and the next input code point is U+0028 LEFT PARENTHESIS (()
        if string.eq_ignore_ascii_case("url") && self.peek_codepoint(0) == Some('(') {
            // consume it
            self.advance(1);

            // 	While the next two input code points are whitespace,
            loop {
                let n1 = self.peek_codepoint(0);
                let n2 = self.peek_codepoint(1);

                match (n1, n2) {
                    (Some(c1), Some(c2)) if is_whitespace(c1) && is_whitespace(c2) => {
                        // consume the next input code point
                        self.advance(1);
                    },
                    _ => break,
                }
            }

            // If the next one or two input code points are U+0022 QUOTATION MARK ("), U+0027 APOSTROPHE ('),
            //  or whitespace followed by U+0022 QUOTATION MARK (") or U+0027 APOSTROPHE (')
            let n1 = self.peek_codepoint(0);
            let n2 = self.peek_codepoint(1);

            if n1 == Some('"')
                || n1 == Some(APOSTROPHE)
                || (n1.is_some()
                    && is_whitespace(n1.unwrap())
                    && (n2 == Some('"') || n2 == Some(APOSTROPHE)))
            {
                // then create a <function-token> with its value set to string and return it
                Token::Function(Cow::Owned(string))
            }
            // Otherwise
            else {
                // consume a url token, and return it.
                self.consume_url_token()
            }
        }
        // Otherwise, if the next input code point is U+0028 LEFT PARENTHESIS (()
        else if self.peek_codepoint(0) == Some('(') {
            // consume it
            self.advance(1);

            // 	Create a <function-token> with its value set to string and return it.
            Token::Function(Cow::Owned(string))
        }
        // Otherwise
        else {
            // create an <ident-token> with its value set to string and return it.
            Token::Ident(Cow::Owned(string))
        }
    }

    /// https://drafts.csswg.org/css-syntax/#consume-a-string-token
    ///
    /// This algorithm may be called with an ending code point, which denotes the code point that ends the string.
    /// If an ending code point is not specified, the current input code point is used.
    /// NOTE: You **need** to provide the ending code point
    fn consume_string_token(&mut self, end_token: char) -> Token<'a> {
        // NOTE this seems a bit silly since we are pretty much guaranteed to use a '"' as the ending code point
        // Initially create a <string-token> with its value set to the empty string.
        let mut string = String::new();

        // Repeatedly consume the next input code point from the stream
        loop {
            match self.next_codepoint() {
                Some(c) if c == end_token => {
                    // Return the <string-token>
                    return Token::String(Cow::Owned(string));
                },
                Some(NEWLINE) => {
                    // This is a parse error.
                    // Reconsume the current input code point
                    self.reconsume();

                    // Create a <bad-string-token>, and return it.
                    return Token::BadString(Cow::Owned(string));
                },
                Some(BACKSLASH) => {
                    match self.peek_codepoint(0) {
                        // If the next input code point is EOF
                        None => {}, // do nothing.
                        // Otherwise, if the next input code point is a newline
                        Some(NEWLINE) => {
                            // consume it
                            self.advance(1);
                        },
                        // Otherwise,
                        Some(_) => {
                            // (the stream starts with a valid escape)

                            // consume an escaped code point and append the returned code point to the <string-token>’s value.
                            string.push(self.consume_escaped_codepoint());
                        },
                    }
                },
                Some(c) => {
                    // Append the current input code point to the <string-token>’s value.
                    string.push(c);
                },
                None => {
                    // This is a parse error.
                    // Return the <string-token>.
                },
            }
        }
    }

    fn advance(&mut self, n: usize) {
        self.position += n;
    }

    /// https://drafts.csswg.org/css-syntax/#consume-token
    pub fn next_token(&mut self) -> Token<'a> {
        // TODO
        // Consume comments.

        // Consume the next input code point.
        match self.next_codepoint() {
            Some(NEWLINE | TAB | WHITESPACE) => {
                // Consume as much whitespace as possible.
                self.consume_whitespace();

                // Return a <whitespace-token>.
                Token::Whitespace
            },

            Some('"') => {
                // Consume a string token and return it.
                self.consume_string_token('"')
            },

            Some('#') => {
                match self.peek_codepoint(0) {
                    // If the next input code point is an ident code point or the next two input code points are a valid escape, then:
                    Some(c) if is_ident_code_point(c) || self.is_valid_escape() => {
                        // Create a <hash-token>.
                        let mut hash_flag = HashFlag::Unrestricted;

                        // If the next 3 input code points would start an ident sequence, set the <hash-token>’s type flag to "id".
                        if self.is_valid_ident_start() {
                            hash_flag = HashFlag::Id;
                        }

                        // Consume an ident sequence, and set the <hash-token>’s value to the returned string.
                        let value = self.consume_ident_sequence();

                        // Return the <hash-token>.
                        Token::Hash(Cow::Owned(value), hash_flag)
                    },
                    // Otherwise
                    _ => {
                        // return a <delim-token> with its value set to the current input code point.
                        Token::Delim('#')
                    },
                }
            },

            Some(APOSTROPHE) => {
                // Consume a string token and return it.
                self.consume_string_token(APOSTROPHE)
            },

            Some('(') => {
                // Return a <(-token>.
                Token::ParenthesisOpen
            },

            Some(')') => {
                // Return a <)-token>.
                Token::ParenthesisClose
            },

            Some('+') => {
                // If the input stream starts with a number
                if self.is_valid_number_start() {
                    // reconsume the current input code point
                    self.reconsume();

                    // consume a numeric token, and return it
                    self.consume_numeric_token()
                }
                // Otherwise,
                else {
                    // return a <delim-token> with its value set to the current input code point.
                    Token::Delim('+')
                }
            },

            Some(',') => {
                // Return a <comma-token>.
                Token::Comma
            },

            Some('-') => {
                // If the input stream starts with a number
                if self.is_valid_number_start() {
                    // reconsume the current input code point
                    self.reconsume();

                    // consume a numeric token, and return it.
                    self.consume_numeric_token()
                }
                // Otherwise, if the next 2 input code points are U+002D HYPHEN-MINUS U+003E GREATER-THAN SIGN (->)
                else if self.peek_codepoint(0) == Some('-') && self.peek_codepoint(1) == Some('>')
                {
                    // consume them
                    self.advance(2);

                    // and return a <CDC-token>.
                    Token::CommentDeclarationClose
                }
                // Otherwise, if the input stream starts with an ident sequence
                else if self.is_valid_ident_start() {
                    // reconsume the current input code point
                    self.reconsume();

                    // consume an ident-like token, and return it.
                    self.consume_ident_like_token()
                }
                // Otherwise,
                else {
                    // return a <delim-token> with its value set to the current input code point.
                    Token::Delim('-')
                }
            },

            Some('.') => {
                // If the input stream starts with a number
                if self.is_valid_number_start() {
                    // reconsume the current input code point
                    self.reconsume();

                    // consume a numeric token, and return it.
                    self.consume_numeric_token()
                }
                // Otherwise,
                else {
                    // return a <delim-token> with its value set to the current input code point.
                    Token::Delim('.')
                }
            },

            Some(':') => {
                // Return a <colon-token>.
                Token::Colon
            },

            Some(';') => {
                // Return a <semicolon-token>.
                Token::Semicolon
            },

            Some('<') => {
                // If the next 3 input code points are U+0021 EXCLAMATION MARK U+002D HYPHEN-MINUS U+002D HYPHEN-MINUS (!--)
                if self.peek_codepoint(0) == Some('!')
                    && self.peek_codepoint(1) == Some('-')
                    && self.peek_codepoint(2) == Some('-')
                {
                    //  consume them
                    self.advance(3);

                    // and return a <CDO-token>.
                    Token::CommentDeclarationOpen
                }
                // Otherwise,
                else {
                    // return a <delim-token> with its value set to the current input code point.
                    Token::Delim('<')
                }
            },

            Some('@') => {
                // If the next 3 input code points would start an ident sequence
                if self.is_valid_ident_start() {
                    // consume an ident sequence,
                    let value = self.consume_ident_sequence();

                    // create an <at-keyword-token> with its value set to the returned value, and return it.
                    Token::AtKeyword(Cow::Owned(value))
                }
                // Otherwise,
                else {
                    // return a <delim-token> with its value set to the current input code point.
                    Token::Delim('@')
                }
            },

            Some('[') => {
                // Return a <[-token>.
                Token::BracketOpen
            },

            Some(BACKSLASH) => {
                // If the input stream starts with a valid escape
                if self.is_valid_escape() {
                    // reconsume the current input code point,
                    self.reconsume();

                    // consume an ident-like token, and return it.
                    self.consume_ident_like_token()
                }
                // Otherwise,
                else {
                    // this is a parse error.
                    // Return a <delim-token> with its value set to the current input code point.
                    Token::Delim(BACKSLASH)
                }
            },

            Some(']') => {
                // Return a <]-token>.
                Token::BracketClose
            },

            Some('{') => {
                // Return a <{-token>.
                Token::CurlyBraceOpen
            },

            Some('}') => {
                // Return a <}-token>.
                Token::CurlyBraceClose
            },

            Some('0'..='9') => {
                // Reconsume the current input code point
                self.reconsume();

                // consume a numeric token, and return it.
                self.consume_numeric_token()
            },

            Some(c) if is_ident_start_code_point(c) => {
                // Reconsume the current input code point
                self.reconsume();

                // consume an ident-like token, and return it.
                self.consume_ident_like_token()
            },

            None => Token::EOF,

            Some(c) => {
                // Return a <delim-token> with its value set to the current input code point.
                Token::Delim(c)
            },
        }
    }
}

/// https://drafts.csswg.org/css-syntax/#whitespace
#[inline]
fn is_whitespace(c: char) -> bool {
    matches!(c, NEWLINE | TAB | WHITESPACE)
}

/// https://drafts.csswg.org/css-syntax/#non-ascii-ident-code-point
#[inline]
fn is_non_ascii_ident_code_point(c: char) -> bool {
    matches!(c, '\u{00B7}' | '\u{00C0}'..'\u{00D6}'
        | '\u{00D8}'..'\u{00F6}'
        | '\u{00F8}'..'\u{037D}'
        | '\u{037F}'..'\u{1FFF}'
        | '\u{200C}' | '\u{200D}' | '\u{203F}' | '\u{2040}'
        | '\u{2070}'..'\u{218F}'
        | '\u{2C00}'..'\u{2FEF}'
        | '\u{3001}'..'\u{D7FF}'
        | '\u{F900}'..'\u{FDCF}'
        | '\u{FDF0}'..'\u{FFFD}'
        | '\u{10000}'..)
}

/// https://drafts.csswg.org/css-syntax/#ident-start-code-point
#[inline]
fn is_ident_start_code_point(c: char) -> bool {
    matches!(c, 'a'..='z' | 'A'..='Z' | '_') || is_non_ascii_ident_code_point(c)
}

/// https://drafts.csswg.org/css-syntax/#ident-code-point
#[inline]
fn is_ident_code_point(c: char) -> bool {
    matches!(c, '-' | '0'..='9') || is_ident_start_code_point(c)
}
