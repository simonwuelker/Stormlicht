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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Number {
    Integer(i32),
    Number(f32),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Token<'a> {
    Ident(Cow<'a, str>),
    AtKeyword(Cow<'a, str>),
    String(Cow<'a, str>),
    BadString(Cow<'a, str>),
    BadURI(Cow<'a, str>),
    Hash(Cow<'a, str>, HashFlag),
    Number(Number),
    Percentage(Number),
    Dimension(Number, Cow<'a, str>),
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
    Function(Cow<'a, str>),
    Comma,
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

    /// Get the current state of the [Tokenizer].
    ///
    /// This can later be used to "reset" it using [set_state](Tokenizer::set_state).
    /// Note that this causes the same source string to be retokenizer a second time.
    /// Since this method is mostly used by the [Parser](crate::parser::Parser) to parse **small** optional tokens,
    /// this is not expected to be a problem, but should be kept in mind anyways.
    pub fn state(&self) -> usize {
        self.position
    }

    /// Set the state of the [Tokenizer]
    ///
    /// Valid states should be obtained from [state](Tokenizer::state).
    pub fn set_state(&mut self, position: usize) {
        self.position = position;
    }

    fn reconsume(&mut self) {
        self.position -= 1;
    }

    fn peek_codepoint(&self, n: usize) -> Option<char> {
        self.source.chars().nth(self.position + n)
    }

    /// <https://drafts.csswg.org/css-syntax/#check-if-two-code-points-are-a-valid-escape>
    fn is_valid_escape_start(&self) -> bool {
        is_valid_escape(self.peek_codepoint(0), self.peek_codepoint(1))
    }

    /// <https://drafts.csswg.org/css-syntax/#check-if-three-code-points-would-start-an-ident-sequence>
    fn is_valid_ident_start(&self) -> bool {
        // Look at the first code point:
        match self.peek_codepoint(0) {
            Some('-') => {
                let n2 = self.peek_codepoint(1);

                // If the second code point is an ident-start code point or a U+002D HYPHEN-MINUS
                // 	or the second and third code points are a valid escape
                // 	return true. Otherwise, return false.
                (n2.is_some() && (is_ident_start_code_point(n2.unwrap()) || n2.unwrap() == '-'))
                    || is_valid_escape(n2, self.peek_codepoint(2))
            },
            Some(BACKSLASH) => {
                // If the first and second code points are a valid escape, return true. Otherwise, return false.
                is_valid_escape(self.peek_codepoint(1), self.peek_codepoint(2))
            },
            Some(c) if is_ident_start_code_point(c) => {
                // Return true.
                true
            },
            _ => {
                // Return false.
                false
            },
        }
    }

    /// <https://drafts.csswg.org/css-syntax/#check-if-three-code-points-would-start-a-number>
    #[allow(clippy::needless_bool, clippy::if_same_then_else)] // spec things...
    fn is_valid_number_start(&self) -> bool {
        // Look at the first code point:
        match self.peek_codepoint(0) {
            Some('+' | '-') => {
                // If the second code point is a digit,
                if matches!(self.peek_codepoint(1), Some('0'..='9')) {
                    // return true.
                    true
                }
                // Otherwise, if the second code point is a U+002E FULL STOP (.) and the third code point is a digit,
                else if self.peek_codepoint(1) == Some('.')
                    && matches!(self.peek_codepoint(2), Some('0'..='9'))
                {
                    // return true.
                    true
                }
                // Otherwise
                else {
                    // return false.
                    false
                }
            },
            Some('.') => {
                // If the second code point is a digit, return true. Otherwise, return false.
                matches!(self.peek_codepoint(1), Some('0'..='9'))
            },
            Some('0'..='9') => {
                // return true.
                true
            },
            _ => {
                // Return false.
                false
            },
        }
    }

    #[inline]
    fn current_position(&self) -> usize {
        self.position
    }

    #[inline]
    fn next_codepoint(&mut self) -> Option<char> {
        let c = self.source.chars().nth(self.position);
        self.advance(1);
        c
    }

    #[inline]
    fn consume_whitespace(&mut self) {
        while matches!(self.peek_codepoint(0), Some(NEWLINE | TAB | WHITESPACE)) {
            self.advance(1);
        }
    }

    /// <https://drafts.csswg.org/css-syntax/#consume-an-ident-sequence>
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
                    else if self.is_valid_escape_start() {
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

    /// <https://drafts.csswg.org/css-syntax-3/#consume-escaped-code-point>
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
                log::warn!(target: "css", "Parse Error: EOF in escaped codepoint");

                // Return U+FFFD REPLACEMENT CHARACTER (�).
                REPLACEMENT
            },
            Some(c) => {
                // Return the current input code point.
                c
            },
        }
    }

    /// <https://drafts.csswg.org/css-syntax/#consume-a-number>
    fn consume_number(&mut self) -> Number {
        // Initially set type to "integer". Let repr be the empty string.
        let mut is_integer = true;

        // NOTE we keep track of repr by remembering the starting position and slicing the source string.
        // This avoids unnecessary heap allocations
        let start = self.current_position();

        // If the next input code point is U+002B PLUS SIGN (+) or U+002D HYPHEN-MINUS (-),
        if matches!(self.peek_codepoint(0), Some('+' | '-')) {
            // consume it and append it to repr.
            self.advance(1);
        }

        // While the next input code point is a digit,
        while matches!(self.peek_codepoint(0), Some('0'..='9')) {
            // consume it and append it to repr.
            self.advance(1);
        }

        // If the next 2 input code points are U+002E FULL STOP (.) followed by a digit, then:
        if self.peek_codepoint(0) == Some('.') && matches!(self.peek_codepoint(1), Some('0'..='9'))
        {
            // Consume them.
            // Append them to repr.
            self.advance(2);

            // Set type to "number".
            is_integer = false;

            // While the next input code point is a digit,
            while matches!(self.peek_codepoint(0), Some('0'..='9')) {
                // consume it and append it to repr.
                self.advance(1);
            }

            // If the next 2 or 3 input code points are U+0045 LATIN CAPITAL LETTER E (E) or U+0065 LATIN SMALL LETTER E (e),
            // optionally followed by U+002D HYPHEN-MINUS (-) or U+002B PLUS SIGN (+), followed by a digit, then:
            if matches!(self.peek_codepoint(0), Some('e' | 'E')) {
                if matches!(self.peek_codepoint(1), Some('0'..='9')) {
                    // Consume them.
                    // Append them to repr.
                    self.advance(2);

                    // Set type to "number".
                    is_integer = true;

                    // While the next input code point is a digit,
                    while matches!(self.peek_codepoint(0), Some('0'..='9')) {
                        // consume it and append it to repr.
                        self.advance(1);
                    }
                } else if matches!(
                    (self.peek_codepoint(1), self.peek_codepoint(2)),
                    (Some('+' | '-'), Some('0'..='9'))
                ) {
                    // Consume them.
                    // Append them to repr.
                    self.advance(3);

                    // Set type to "number".
                    is_integer = true;

                    // While the next input code point is a digit,
                    while matches!(self.peek_codepoint(0), Some('0'..='9')) {
                        // consume it and append it to repr.
                        self.advance(1);
                    }
                }
            }
        }

        // Convert repr to a number, and set the value to the returned value.
        // Return value and type.
        let end = self.current_position();
        if is_integer {
            Number::Integer(self.source[start..end].parse().unwrap())
        } else {
            Number::Number(self.source[start..end].parse().unwrap())
        }
    }

    /// <https://drafts.csswg.org/css-syntax/#consume-a-numeric-token>
    fn consume_numeric_token(&mut self) -> Token<'a> {
        // Consume a number and let number be the result.
        let number = self.consume_number();

        // If the next 3 input code points would start an ident sequence, then:
        if self.is_valid_ident_start() {
            // Create a <dimension-token> with the same value and type flag as number, and a unit set initially to the empty string.

            // Consume an ident sequence. Set the <dimension-token>’s unit to the returned value.
            let unit = self.consume_ident_sequence();

            // Return the <dimension-token>.
            Token::Dimension(number, Cow::Owned(unit))
        }
        // Otherwise, if the next input code point is U+0025 PERCENTAGE SIGN (%)
        else if self.peek_codepoint(0) == Some('%') {
            //	consume it.
            self.advance(1);

            // Create a <percentage-token> with the same value as number, and return it.
            Token::Percentage(number)
        }
        // Otherwise,
        else {
            // create a <number-token> with the same value and type flag as number, and return it.
            Token::Number(number)
        }
    }

    /// <https://drafts.csswg.org/css-syntax/#consume-the-remnants-of-a-bad-url>
    fn consume_remnants_of_a_bad_url(&mut self) {
        // Repeatedly consume the next input code point from the stream:
        loop {
            let n = self.next_codepoint();

            if matches!(n, Some('(') | None) {
                // Return.
                return;
            } else if self.is_valid_escape_start() {
                // Consume an escaped code point.
                self.consume_escaped_codepoint();

                // This allows an escaped right parenthesis ("\)") to be encountered without ending the <bad-url-token>. This is otherwise identical to the "anything else" clause.
            } else {
                // Do nothing.
            }
        }
    }

    /// <https://drafts.csswg.org/css-syntax/#consume-a-url-token>
    fn consume_url_token(&mut self) -> Token<'a> {
        // Initially create a <url-token> with its value set to the empty string.
        let mut value = String::new();

        // Consume as much whitespace as possible.
        self.consume_whitespace();

        // Repeatedly consume the next input code point from the stream:
        loop {
            match self.next_codepoint() {
                Some(')') => {
                    // Return the <url-token>.
                    return Token::URI(Cow::Owned(value));
                },
                None => {
                    // This is a parse error.
                    log::warn!(target: "css", "Parse Error: EOF in URL token");

                    // Return the <url-token>.
                    return Token::URI(Cow::Owned(value));
                },
                Some(c) if is_whitespace(c) => {
                    // Consume as much whitespace as possible
                    self.consume_whitespace();

                    //  If the next input code point is U+0029 RIGHT PARENTHESIS ()) or EOF
                    if matches!(self.peek_codepoint(0), Some(')') | None) {
                        // consume it
                        self.advance(1);

                        // and return the <url-token>
                        // (if EOF was encountered, this is a parse error)
                        if self.peek_codepoint(0).is_none() {
                            log::warn!(target: "css", "Parse Error: EOF in URL token");
                        }
                        return Token::URI(Cow::Owned(value));
                    }
                    // otherwise,
                    else {
                        // consume the remnants of a bad url
                        self.consume_remnants_of_a_bad_url();

                        // create a <bad-url-token>, and return it.
                        return Token::BadURI(Cow::Owned(value));
                    }
                },
                Some(
                    c @ ('"'
                    | APOSTROPHE
                    | '('
                    | '\x00'..='\x08'
                    | '\x0b'
                    | '\x0e'..='\x1f'
                    | '\x7f'),
                ) => {
                    // This is a parse error.
                    log::warn!(target: "css", "Parse Error: Illegal character {c:?} in URL token");

                    // Consume the remnants of a bad url
                    self.consume_remnants_of_a_bad_url();

                    // create a <bad-url-token>, and return it.
                    return Token::BadURI(Cow::Owned(value));
                },
                Some(BACKSLASH) => {
                    // If the stream starts with a valid escape
                    if self.is_valid_escape_start() {
                        // consume an escaped code point
                        let c = self.consume_escaped_codepoint();

                        // and append the returned code point to the <url-token>’s value.
                        value.push(c);
                    }
                    // Otherwise,
                    else {
                        // This is a parse error.
                        log::warn!(target: "css", "Parse Error: Backslash character is not a valid escape start");

                        // Consume the remnants of a bad url
                        self.consume_remnants_of_a_bad_url();

                        // create a <bad-url-token>, and return it.
                        return Token::BadURI(Cow::Owned(value));
                    }
                },
                Some(c) => {
                    // Append the current input code point to the <url-token>’s value.
                    value.push(c);
                },
            }
        }
    }

    /// <https://drafts.csswg.org/css-syntax/#consume-an-ident-like-token>
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

    /// <https://drafts.csswg.org/css-syntax/#consume-a-string-token>
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
                    log::warn!(target: "css", "Parse Error: Newline in string token");

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
                    log::warn!(target: "css", "Parse Error: EOF in string token");

                    // Return the <string-token>.
                    return Token::String(Cow::Owned(string));
                },
            }
        }
    }

    // <https://drafts.csswg.org/css-syntax/#consume-comment>
    fn consume_comments(&mut self) {
        // If the next two input code point are U+002F SOLIDUS (/) followed by a U+002A ASTERISK (*),
        while self.peek_codepoint(0) == Some('/') && self.peek_codepoint(1) == Some('*') {
            // consume them
            self.advance(2);

            // and all following code points up to and including the first U+002A ASTERISK (*) followed by a U+002F SOLIDUS (/), or up to an EOF code point.
            loop {
                match self.next_codepoint() {
                    Some('*') => {
                        if self.next_codepoint() == Some('/') {
                            break;
                        }
                    },
                    None => {
                        log::warn!(target: "css", "Parse Error: EOF in comment");
                        break;
                    },
                    _ => {},
                }
            }
            // Return to the start of this step.
        }

        // If the preceding paragraph ended by consuming an EOF code point, this is a parse error.
        // NOTE: we report the error above

        // Return nothing.
    }

    fn advance(&mut self, n: usize) {
        self.position += n;
    }

    /// Read the next token from the input stream
    ///
    /// # Specification
    /// <https://drafts.csswg.org/css-syntax/#consume-token>
    pub fn next_token(&mut self) -> Option<Token<'a>> {
        // Consume comments.
        self.consume_comments();

        // Consume the next input code point.
        match self.next_codepoint() {
            Some(NEWLINE | TAB | WHITESPACE) => {
                // Consume as much whitespace as possible.
                self.consume_whitespace();

                // Return a <whitespace-token>.
                Some(Token::Whitespace)
            },

            Some('"') => {
                // Consume a string token and return it.
                Some(self.consume_string_token('"'))
            },

            Some('#') => {
                match self.peek_codepoint(0) {
                    // If the next input code point is an ident code point or the next two input code points are a valid escape, then:
                    Some(c) if is_ident_code_point(c) || self.is_valid_escape_start() => {
                        // Create a <hash-token>.
                        let mut hash_flag = HashFlag::Unrestricted;

                        // If the next 3 input code points would start an ident sequence, set the <hash-token>’s type flag to "id".
                        if self.is_valid_ident_start() {
                            hash_flag = HashFlag::Id;
                        }

                        // Consume an ident sequence, and set the <hash-token>’s value to the returned string.
                        let value = self.consume_ident_sequence();

                        // Return the <hash-token>.
                        Some(Token::Hash(Cow::Owned(value), hash_flag))
                    },
                    // Otherwise
                    _ => {
                        // return a <delim-token> with its value set to the current input code point.
                        Some(Token::Delim('#'))
                    },
                }
            },

            Some(APOSTROPHE) => {
                // Consume a string token and return it.
                Some(self.consume_string_token(APOSTROPHE))
            },

            Some('(') => {
                // Return a <(-token>.
                Some(Token::ParenthesisOpen)
            },

            Some(')') => {
                // Return a <)-token>.
                Some(Token::ParenthesisClose)
            },

            Some('+') => {
                // If the input stream starts with a number
                if self.is_valid_number_start() {
                    // reconsume the current input code point
                    self.reconsume();

                    // consume a numeric token, and return it
                    Some(self.consume_numeric_token())
                }
                // Otherwise,
                else {
                    // return a <delim-token> with its value set to the current input code point.
                    Some(Token::Delim('+'))
                }
            },

            Some(',') => {
                // Return a <comma-token>.
                Some(Token::Comma)
            },

            Some('-') => {
                // If the input stream starts with a number
                if self.is_valid_number_start() {
                    // reconsume the current input code point
                    self.reconsume();

                    // consume a numeric token, and return it.
                    Some(self.consume_numeric_token())
                }
                // Otherwise, if the next 2 input code points are U+002D HYPHEN-MINUS U+003E GREATER-THAN SIGN (->)
                else if self.peek_codepoint(0) == Some('-') && self.peek_codepoint(1) == Some('>')
                {
                    // consume them
                    self.advance(2);

                    // and return a <CDC-token>.
                    Some(Token::CommentDeclarationClose)
                }
                // Otherwise, if the input stream starts with an ident sequence
                else if self.is_valid_ident_start() {
                    // reconsume the current input code point
                    self.reconsume();

                    // consume an ident-like token, and return it.
                    Some(self.consume_ident_like_token())
                }
                // Otherwise,
                else {
                    // return a <delim-token> with its value set to the current input code point.
                    Some(Token::Delim('-'))
                }
            },

            Some('.') => {
                // If the input stream starts with a number
                if self.is_valid_number_start() {
                    // reconsume the current input code point
                    self.reconsume();

                    // consume a numeric token, and return it.
                    Some(self.consume_numeric_token())
                }
                // Otherwise,
                else {
                    // return a <delim-token> with its value set to the current input code point.
                    Some(Token::Delim('.'))
                }
            },

            Some(':') => {
                // Return a <colon-token>.
                Some(Token::Colon)
            },

            Some(';') => {
                // Return a <semicolon-token>.
                Some(Token::Semicolon)
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
                    Some(Token::CommentDeclarationOpen)
                }
                // Otherwise,
                else {
                    // return a <delim-token> with its value set to the current input code point.
                    Some(Token::Delim('<'))
                }
            },

            Some('@') => {
                // If the next 3 input code points would start an ident sequence
                if self.is_valid_ident_start() {
                    // consume an ident sequence,
                    let value = self.consume_ident_sequence();

                    // create an <at-keyword-token> with its value set to the returned value, and return it.
                    Some(Token::AtKeyword(Cow::Owned(value)))
                }
                // Otherwise,
                else {
                    // return a <delim-token> with its value set to the current input code point.
                    Some(Token::Delim('@'))
                }
            },

            Some('[') => {
                // Return a <[-token>.
                Some(Token::BracketOpen)
            },

            Some(BACKSLASH) => {
                // If the input stream starts with a valid escape
                if self.is_valid_escape_start() {
                    // reconsume the current input code point,
                    self.reconsume();

                    // consume an ident-like token, and return it.
                    Some(self.consume_ident_like_token())
                }
                // Otherwise,
                else {
                    // this is a parse error.
                    log::warn!(target: "css", "Parse Error: Backslash character is not a valid escape start");

                    // Return a <delim-token> with its value set to the current input code point.
                    Some(Token::Delim(BACKSLASH))
                }
            },

            Some(']') => {
                // Return a <]-token>.
                Some(Token::BracketClose)
            },

            Some('{') => {
                // Return a <{-token>.
                Some(Token::CurlyBraceOpen)
            },

            Some('}') => {
                // Return a <}-token>.
                Some(Token::CurlyBraceClose)
            },

            Some('0'..='9') => {
                // Reconsume the current input code point
                self.reconsume();

                // consume a numeric token, and return it.
                Some(self.consume_numeric_token())
            },

            Some(c) if is_ident_start_code_point(c) => {
                // Reconsume the current input code point
                self.reconsume();

                // consume an ident-like token, and return it.
                Some(self.consume_ident_like_token())
            },

            Some(c) => {
                // Return a <delim-token> with its value set to the current input code point.
                Some(Token::Delim(c))
            },

            None => None,
        }
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

/// <https://drafts.csswg.org/css-syntax/#whitespace>
#[inline]
fn is_whitespace(c: char) -> bool {
    matches!(c, NEWLINE | TAB | WHITESPACE)
}

/// <https://drafts.csswg.org/css-syntax/#non-ascii-ident-code-point>
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

/// <https://drafts.csswg.org/css-syntax/#ident-start-code-point>
#[inline]
fn is_ident_start_code_point(c: char) -> bool {
    matches!(c, 'a'..='z' | 'A'..='Z' | '_') || is_non_ascii_ident_code_point(c)
}

/// <https://drafts.csswg.org/css-syntax/#ident-code-point>
#[inline]
fn is_ident_code_point(c: char) -> bool {
    matches!(c, '-' | '0'..='9') || is_ident_start_code_point(c)
}

/// <https://drafts.csswg.org/css-syntax/#check-if-three-code-points-would-start-an-ident-sequence>
#[inline]
fn is_valid_escape(c1: Option<char>, c2: Option<char>) -> bool {
    // If the first code point is not U+005C REVERSE SOLIDUS (\)
    if c1 != Some(BACKSLASH) {
        // return false.
        false
    } else {
        // Otherwise, if the second code point is a newline return false.
        // Otherwise return true.
        c2 != Some(NEWLINE)
    }
}
