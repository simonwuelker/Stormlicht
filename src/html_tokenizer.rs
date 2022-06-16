const UNICODE_REPLACEMENT: char = '\u{FFFD}';

#[derive(Debug)]
pub enum TokenizerState {
    DataState,
    RCDATAState,
    RAWTEXTState,
    ScriptDataState,
    PLAINTEXTState,
    TagOpenState,
    EndTagOpenState,
    TagNameState,
    RCDATALessThanSignState,
    RCDATAEndTagOpenState,
    RCDATAEndTagNameState,
    RAWTEXTLessThanSignState,
    RAWTEXTEndTagOpenState,
    RAWTEXTEndTagNameState,
    ScriptDataLessThanSignState,
    ScriptDataEndTagOpenState,
    ScriptDataEndTagNameState,
    ScriptDataEscapeStartState,
    ScriptDataEscapeStartDashState,
    ScriptDataEscapedState,
    ScriptDataEscapedDashState,
    ScriptDataEscapedDashDashState,
    ScriptDataEscapedLessThanSignState,
    ScriptDataEscapedEndTagOpenState,
    ScriptDataEscapedEndTagNameState,
    ScriptDataDoubleEscapeStartState,
    ScriptDataDoubleEscapedState,
    ScriptDataDoubleEscapedDashState,
    ScriptDataDoubleEscapedDashDashState,
    ScriptDataDoubleEscapedLessThanSignState,
    ScriptDataDoubleEscapeEndState,
    BeforeAttributeNameState,
    AttributeNameState,
    AfterAttributeNameState,
    BeforeAttributeValueState,
    AttributeValueDoublequotedState,
    AttributeValueSinglequotedState,
    AttributeValueUnquotedState,
    AfterAttributeValueQuotedState,
    SelfClosingStartTagState,
    BogusCommentState,
    MarkupDeclarationOpenState,
    CommentStartState,
    CommentStartDashState,
    CommentState,
    CommentLessThanSignState,
    CommentLessThanSignBangState,
    CommentLessThanSignBangDashState,
    CommentLessThanSignBangDashDashState,
    CommentEndDashState,
    CommentEndState,
    CommentEndBangState,
    DOCTYPEState,
    BeforeDOCTYPENameState,
    DOCTYPENameState,
    AfterDOCTYPENameState,
    AfterDOCTYPEPublicKeywordState,
    BeforeDOCTYPEPublicIdentifierState,
    DOCTYPEPublicIdentifierDoublequotedState,
    DOCTYPEPublicIdentifierSinglequotedState,
    AfterDOCTYPEPublicIdentifierState,
    BetweenDOCTYPEPublicAndSystemIdentifiersState,
    AfterDOCTYPESystemKeywordState,
    BeforeDOCTYPESystemIdentifierState,
    DOCTYPESystemIdentifierDoublequotedState,
    DOCTYPESystemIdentifierSinglequotedState,
    AfterDOCTYPESystemIdentifierState,
    BogusDOCTYPEState,
    CDATASectionState,
    CDATASectionBracketState,
    CDATASectionEndState,
    CharacterReferenceState,
    NamedCharacterReferenceState,
    AmbiguousAmpersandState,
    NumericCharacterReferenceState,
    HexadecimalCharacterReferenceStartState,
    DecimalCharacterReferenceStartState,
    HexadecimalCharacterReferenceState,
    DecimalCharacterReferenceState,
    NumericCharacterReferenceEndState,
}

#[derive(Debug)]
pub enum Token {
    DOCTYPE {
        name: Option<String>,
        public_ident: Option<String>,
        system_ident: Option<String>,
        force_quirks: bool,
    }, 
    StartTag {
        name: String,
        self_closing: bool,
        attributes: Vec<(String, String)>,
    }, 
    EndTag {
        name: String,
        self_closing: bool,
        attributes: Vec<(String, String)>,
    }, 
    Comment(String), 
    Character(char), 
    EOF,
}

pub struct Tokenizer<'source> {
    source: &'source str,
    state: TokenizerState,
    /// Used by [TokenizerState::CharacterReferenceState]
    return_state: Option<TokenizerState>,
    current_token: Option<Token>,
    last_emitted_start_token_name: Option<String>,
    buffer: Option<String>,
    ptr: usize,
}


impl<'source> Tokenizer<'source> {
    pub fn new(source: &'source str) -> Self {
        Self {
            source: source,
            state: TokenizerState::DataState,
            return_state: None,
            current_token: None,
            last_emitted_start_token_name: None,
            buffer: None,
            ptr: 0,
        }
    }

    fn emit(&mut self, token: Token) {
        if let Token::StartTag{ ref name, .. } = token {
            self.last_emitted_start_token_name = Some(name.clone());
        }
        println!("emitting {:?}", token);
    }

    fn emit_current_token(&mut self) {
        let current_token = self.current_token.take().unwrap();
        self.emit(current_token);
    }

    /// Whether the current token is an [Token::EndTag] token whose name matches
    /// the name of the last [Token::StartTag] token that was emitted.
    fn is_appropriate_end_token(&self) -> bool {
        match (&self.last_emitted_start_token_name, &self.current_token) {
            (Some(start_name), Some(current_token)) => {
                match current_token {
                    Token::EndTag { name, .. } => {
                        name == start_name
                    },
                    _ => false,
                }
            },
            _ => false,
        }
    }

    fn add_to_current_tag_name(&mut self, c: char) {
        match &mut self.current_token {
            Some(ref mut token) => {
                match token {
                    Token::StartTag { ref mut name, .. } | Token::EndTag { ref mut name, .. } => {
                        name.push(c);
                    },
                    _ => panic!("Trying to write {} to a tag name but the current tag is unsuitable ({:?})", c, token),
                }
            },
            None => {
                panic!("Trying to write {} to a tag name, but no tag is present", c)
            }

        }
    }

    fn add_to_buffer(&mut self, c: char) {
        match &mut self.buffer {
            Some(ref mut buffer) => {
                buffer.push(c);
            },
            None => panic!("Trying to write {} to self.buffer but self.buffer is None", c),
        }
    }

    fn read_next(&mut self) -> Option<char> {
        let c = self.source[self.ptr..].chars().nth(0);
        self.ptr += 1;
        c
    }


    pub fn step(&mut self) {
        match self.state {
            TokenizerState::DataState => {
                match self.read_next() {
                    Some('&') => {
                        self.return_state = Some(TokenizerState::DataState);
                        self.state = TokenizerState::CharacterReferenceState;
                    },
                    Some('<') => {
                        self.state = TokenizerState::TagOpenState;
                    },
                    Some('\0') => {
                        // Unexpected null character parse error - emit replacement token
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    }
                    Some(c) => {
                        self.emit(Token::Character(c));
                    }
                    None =>  {
                        self.emit(Token::EOF);
                    }
                }
            },
            TokenizerState::RCDATAState => {
                match self.read_next() {
                    Some('&') => {
                        self.return_state = Some(TokenizerState::RCDATAState);
                        self.state = TokenizerState::CharacterReferenceState;
                    },
                    Some('<') => {
                        self.state = TokenizerState::RCDATALessThanSignState;
                    },
                    Some('\0') => {
                        // Unexpected null character parse error - emit replacement token
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    }
                    Some(c) => {
                        self.emit(Token::Character(c));
                    }
                    None =>  {
                        self.emit(Token::EOF);
                    }
                }
            },
            TokenizerState::RAWTEXTState => {
                match self.read_next() {
                    Some('<') => {
                        self.state = TokenizerState::RAWTEXTLessThanSignState;
                    },
                    Some('\0') => {
                        // Unexpected null character parse error - emit replacement token
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    },
                    Some(c) => {
                        self.emit(Token::Character(c));
                    }
                    None => {
                        self.emit(Token::EOF);
                    }
                }
            },
            TokenizerState::ScriptDataState => {
                match self.read_next() {
                    Some('<') => {
                        self.state = TokenizerState::ScriptDataLessThanSignState;
                    }
                    Some('\0') => {
                        // Unexpected null character parse error - emit replacement token
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    }
                    Some(c) => {
                        self.emit(Token::Character(c));
                    }
                    None =>  {
                        self.emit(Token::EOF);
                    }
                }
            },
            TokenizerState::PLAINTEXTState => {
                match self.read_next() {
                    Some('\0') => {
                        // Unexpected null character parse error - emit replacement token
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    }
                    Some(c) => {
                        self.emit(Token::Character(c));
                    }
                    None =>  {
                        self.emit(Token::EOF);
                    }
                }
            },
            TokenizerState::TagOpenState => {
                match self.read_next() {
                    Some('!') => {
                        self.state = TokenizerState::MarkupDeclarationOpenState;
                    },
                    Some('/') => {
                        self.state = TokenizerState::EndTagOpenState;
                    },
                    Some('a'..='z' | 'A'..='Z') => {
                        self.current_token = Some(Token::StartTag {
                            name: String::new(),
                            self_closing: false,
                            attributes: Vec::new(),
                        });
                        self.ptr -= 1; // reconsume the current character
                        self.state = TokenizerState::TagNameState;
                    },
                    Some('?') => {
                        // Unexpected question mark instead of tag name parse error, 
                        self.current_token = Some(Token::Comment(String::new()));
                        self.ptr -= 1; // reconsume current character
                        self.state = TokenizerState::BogusCommentState;
                    },
                    Some(c) => {
                        // invalid-first-character-of-tag-name 
                        self.ptr -= 1; // reconsume current token
                        self.state = TokenizerState::DataState;
                        self.emit(Token::Character('<'));
                    },
                    None => {
                        // eof before tag name parse error
                        self.emit(Token::Character('<'));
                        self.emit(Token::EOF);
                    },
                }
            }
            TokenizerState::EndTagOpenState => {
                match self.read_next() {
                    Some('a'..='z' | 'A'..='Z') => {
                        self.ptr -= 1;
                        self.state = TokenizerState::TagNameState;
                        todo!();
                    },
                    Some('>') => {
                        // missing-end-tag-name parse error
                        self.state = TokenizerState::DataState;
                    },
                    Some(c) => {
                        // invalid-first-character-of-tag-name parse error
                        self.ptr -= 1;
                        self.state = TokenizerState::BogusCommentState;
                        todo!();
                    },
                    None => {
                        // eof-before-tag-name parse error
                        self.emit(Token::Character('<'));
                        self.emit(Token::Character('/'));
                        self.emit(Token::EOF);
                    }
                }
            },
            TokenizerState::TagNameState => {
                match self.read_next() {
                    //       tab       line feed    form feed     space
                    Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}') => {
                        self.state = TokenizerState::BeforeAttributeNameState;
                    },
                    Some('/') => {
                        self.state = TokenizerState::SelfClosingStartTagState;
                    },
                    Some('>') => {
                        self.state = TokenizerState::DataState;
                        let token = self.current_token.take().unwrap();
                        self.emit(token);
                    },
                    Some(mut c @ 'A'..='Z') => {
                        c.make_ascii_lowercase();
                        self.add_to_current_tag_name(c);
                    },
                    Some('\0') => {
                        // unexpected null character parse error
                        self.add_to_current_tag_name(UNICODE_REPLACEMENT);
                    },
                    Some(c) => {
                        self.add_to_current_tag_name(c);
                    },
                    None => {
                        self.emit(Token::EOF);
                    }
                }
            },
            TokenizerState::RCDATALessThanSignState => {
                match self.read_next() {
                    Some('/') => {
                        self.buffer = Some(String::new());
                        self.state = TokenizerState::RCDATAEndTagOpenState;
                    },
                    _ => {
                        self.emit(Token::Character('<'));
                        self.ptr -= 1;
                        self.state = TokenizerState::RCDATAState;
                    },
                }
            },
            TokenizerState::RCDATAEndTagOpenState => {
                match self.read_next() {
                    Some('a'..='z' | 'A'..='Z') => {
                        self.current_token = Some(Token::EndTag {
                            name: String::new(),
                            self_closing: false,
                            attributes: Vec::new(),
                        });
                        self.ptr -= 1;
                        self.state = TokenizerState::RCDATAEndTagNameState;
                    },
                    _ => {
                        self.emit(Token::Character('<'));
                        self.emit(Token::Character('/'));
                        self.ptr -= 1;
                        self.state = TokenizerState::RCDATAState;
                    },
                }
            },
            TokenizerState::RCDATAEndTagNameState => {
                match (self.read_next(), self.is_appropriate_end_token()) {
                    //       tab       line feed    form feed     space
                    (Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}'), true) => {
                        self.state = TokenizerState::BeforeAttributeNameState;
                    },
                    (Some('/'), true) => {
                        self.state = TokenizerState::SelfClosingStartTagState;
                    },
                    (Some('>'), true) => {
                        self.state = TokenizerState::DataState;
                        self.emit_current_token();
                    },
                    (Some(mut c @ 'A'..='Z'), _) => {
                        self.add_to_buffer(c);
                        c.make_ascii_lowercase();
                        self.add_to_current_tag_name(c);

                    },
                    (Some(c @ 'a'..='z'), _) => {
                        self.add_to_buffer(c);
                        self.add_to_current_tag_name(c);
                    },
                    _ => {
                        self.emit(Token::Character('<'));
                        self.emit(Token::Character('/'));
                        for c in self.buffer.take().unwrap().chars() {
                            self.emit(Token::Character(c));
                        }
                        self.ptr -= 1;
                        self.state = TokenizerState::RCDATAState;
                    },
                }
            },
            TokenizerState::RAWTEXTLessThanSignState => {
                match self.read_next() {
                    Some('/') => {
                        self.buffer = Some(String::new());
                        self.state = TokenizerState::RAWTEXTEndTagOpenState;
                    },
                    _ => {
                        self.emit(Token::Character('<'));
                        self.ptr -= 1;
                        self.state = TokenizerState::RAWTEXTState;
                    }
                }
            },
            TokenizerState::RAWTEXTEndTagOpenState => {
                match self.read_next() {
                    Some('a'..='z' | 'A'..='Z') => {
                        self.current_token = Some(Token::EndTag {
                            name: String::new(),
                            self_closing: false,
                            attributes: Vec::new(),
                        });
                        self.ptr -= 1;
                        self.state = TokenizerState::RAWTEXTEndTagNameState;
                    },
                    _ => {
                        self.emit(Token::Character('<'));
                        self.emit(Token::Character('/'));
                        self.ptr -= 1;
                        self.state = TokenizerState::RAWTEXTState;
                    },
                }
            },
            TokenizerState::RAWTEXTEndTagNameState => {
                match (self.read_next(), self.is_appropriate_end_token()) {
                    //       tab       line feed    form feed     space
                    (Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}'), true) => {
                        self.state = TokenizerState::BeforeAttributeNameState;
                    },
                    (Some('/'), true) => {
                        self.state = TokenizerState::SelfClosingStartTagState;
                    },
                    (Some('>'), true) => {
                        self.state = TokenizerState::DataState;
                        self.emit_current_token();
                    },
                    (Some(mut c @ 'A'..='Z'), _) => {
                        self.add_to_buffer(c);
                        c.make_ascii_lowercase();
                        self.add_to_current_tag_name(c);

                    },
                    (Some(c @ 'a'..='z'), _) => {
                        self.add_to_buffer(c);
                        self.add_to_current_tag_name(c);
                    },
                    _ => {
                        self.emit(Token::Character('<'));
                        self.emit(Token::Character('/'));
                        for c in self.buffer.take().unwrap().chars() {
                            self.emit(Token::Character(c));
                        }
                        self.ptr -= 1;
                        self.state = TokenizerState::RAWTEXTState;
                    },
                }
            },
            TokenizerState::ScriptDataLessThanSignState => {
                match self.read_next() {
                    Some('/') => {
                        self.buffer = Some(String::new());
                        self.state = TokenizerState::ScriptDataEndTagOpenState;
                    },
                    Some('!') => {
                        self.state = TokenizerState::ScriptDataEscapeStartState;
                        self.emit(Token::Character('<'));
                        self.emit(Token::Character('!'));
                    },
                    _ => {
                        self.emit(Token::Character('<'));
                        self.ptr -= 1;
                        self.state = TokenizerState::ScriptDataState;
                    },
                }
            },
            TokenizerState::ScriptDataEndTagOpenState => {
                match self.read_next() {
                    Some('a'..='z' | 'A'..='Z') => {
                        self.current_token = Some(Token::EndTag {
                            name: String::new(),
                            self_closing: false,
                            attributes: Vec::new(),
                        });
                        self.ptr -= 1;
                        self.state = TokenizerState::ScriptDataEndTagNameState;
                    },
                    _ => {
                        self.emit(Token::Character('<'));
                        self.emit(Token::Character('/'));
                        self.ptr -= 1;
                        self.state = TokenizerState::ScriptDataState;
                    },
                }
            },
            TokenizerState::ScriptDataEndTagNameState => {
                match (self.read_next(), self.is_appropriate_end_token()) {
                    //       tab       line feed    form feed     space
                    (Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}'), true) => {
                        self.state = TokenizerState::BeforeAttributeNameState;
                    },
                    (Some('/'), true) => {
                        self.state = TokenizerState::SelfClosingStartTagState;
                    },
                    (Some('>'), true) => {
                        self.state = TokenizerState::DataState;
                        self.emit_current_token();
                    },
                    (Some(mut c @ 'A'..='Z'), _) => {
                        self.add_to_buffer(c);
                        c.make_ascii_lowercase();
                        self.add_to_current_tag_name(c);

                    },
                    (Some(c @ 'a'..='z'), _) => {
                        self.add_to_buffer(c);
                        self.add_to_current_tag_name(c);
                    },
                    _ => {
                        self.emit(Token::Character('<'));
                        self.emit(Token::Character('/'));
                        for c in self.buffer.take().unwrap().chars() {
                            self.emit(Token::Character(c));
                        }
                        self.ptr -= 1;
                        self.state = TokenizerState::ScriptDataState;
                    },
                }
            },
            TokenizerState::ScriptDataEscapeStartState => {
                match self.read_next() {
                    Some('-') => {
                        self.state = TokenizerState::ScriptDataEscapeStartDashState;
                        self.emit(Token::Character('-'));
                    },
                    _ => {
                        self.ptr -= 1;
                        self.state = TokenizerState::ScriptDataState;
                    }
                }
            },
            TokenizerState::ScriptDataEscapeStartDashState => {
                match self.read_next() {
                    Some('-') => {
                        self.state = TokenizerState::ScriptDataEscapedDashDashState;
                        self.emit(Token::Character('-'));
                    },
                    _ => {
                        self.ptr -= 1;
                        self.state = TokenizerState::ScriptDataState;
                    }
                }
            },
            TokenizerState::ScriptDataEscapedState => {
                match self.read_next() {
                    Some('-') => {
                        self.state = TokenizerState::ScriptDataEscapedDashState;
                        self.emit(Token::Character('-'));
                    },
                    Some('<') => {
                        self.state = TokenizerState::ScriptDataEscapedLessThanSignState;
                    },
                    Some('\0') => {
                        // unexpected null character parse error
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    },
                    Some(c) => {
                        self.emit(Token::Character(c));
                    },
                    None => {
                        // eof in script html comment like text parse error
                        self.emit(Token::EOF);
                    }
                }
            },
            TokenizerState::ScriptDataEscapedDashState => {
                match self.read_next() {
                    Some('-') => {
                        self.state = TokenizerState::ScriptDataEscapedDashDashState;
                        self.emit(Token::Character('-'));
                    },
                    Some('<') => {
                        self.state = TokenizerState::ScriptDataEscapedLessThanSignState;
                    },
                    Some('\0') => {
                        // unexpected null character parse error
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    },
                    Some(c) => {
                        self.state = TokenizerState::ScriptDataEscapedState;
                        self.emit(Token::Character(c));
                    },
                    None => {
                        // eof in script html comment like text parse error
                        self.emit(Token::EOF);
                    },
                }
            },
            TokenizerState::ScriptDataEscapedDashDashState => {
                match self.read_next() {
                    Some('-') => {
                        self.emit(Token::Character('-'));
                    },
                    Some('<') => {
                        self.state = TokenizerState::ScriptDataEscapedLessThanSignState;
                    },
                    Some('>') => {
                        self.state = TokenizerState::ScriptDataState;
                        self.emit(Token::Character('>'));
                    },
                    Some('\0') => {
                        // unexpected null character parse error
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    },
                    Some(c) => {
                        self.state = TokenizerState::ScriptDataEscapedState;
                        self.emit(Token::Character(c));
                    },
                    None => {
                        // eof in script html comment like text parse error
                        self.emit(Token::EOF);
                    }
                }
            },
            TokenizerState::ScriptDataEscapedLessThanSignState => {
                match self.read_next() {
                    Some('/') => {
                        self.buffer = Some(String::new());
                        self.state = TokenizerState::ScriptDataEscapedEndTagOpenState;
                    },
                    Some('a'..='z' | 'A'..='Z') => {
                        self.buffer = Some(String::new());
                        self.emit(Token::Character('<'));
                        self.ptr -= 1;
                        self.state = TokenizerState::ScriptDataDoubleEscapeStartState;
                    },
                    _ => {
                        self.emit(Token::Character('<'));
                        self.ptr -= 1;
                        self.state = TokenizerState::ScriptDataEscapedState;
                    },
                }
            },
            TokenizerState::ScriptDataEscapedEndTagOpenState => {
                match self.read_next() {
                    Some('a'..='z' | 'A'..='Z') => {
                        self.current_token = Some(Token::EndTag {
                            name: String::new(),
                            self_closing: false,
                            attributes: Vec::new(),
                        });
                        self.ptr -= 1;
                        self.state = TokenizerState::ScriptDataEscapedEndTagNameState;
                    },
                    _ => {
                        self.emit(Token::Character('<'));
                        self.emit(Token::Character('/'));
                        self.ptr -= 1;
                        self.state = TokenizerState::ScriptDataEscapedState;
                    },
                }
            },
            TokenizerState::ScriptDataEscapedEndTagNameState => {
                match (self.read_next(), self.is_appropriate_end_token()) {
                    //       tab       line feed    form feed     space
                    (Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}'), true) => {
                        self.state = TokenizerState::BeforeAttributeNameState;
                    },
                    (Some('/'), true) => {
                        self.state = TokenizerState::SelfClosingStartTagState;
                    },
                    (Some('>'), true) => {
                        self.state = TokenizerState::DataState;
                        self.emit_current_token();
                    },
                    (Some(mut c @ 'A'..='Z'), _) => {
                        self.add_to_buffer(c);
                        c.make_ascii_lowercase();
                        self.add_to_current_tag_name(c);

                    },
                    (Some(c @ 'a'..='z'), _) => {
                        self.add_to_buffer(c);
                        self.add_to_current_tag_name(c);
                    },
                    _ => {
                        self.emit(Token::Character('<'));
                        self.emit(Token::Character('/'));
                        for c in self.buffer.take().unwrap().chars() {
                            self.emit(Token::Character(c));
                        }
                        self.ptr -= 1;
                        self.state = TokenizerState::ScriptDataEscapedState;
                    },
                }
            },
            TokenizerState::ScriptDataDoubleEscapeStartState => {
                match self.read_next() {
                    //             tab       line feed    form feed     space
                    Some(c @ ('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' | '/' | '>')) => {
                        if self.buffer.contains(&"script") {
                            self.state = TokenizerState::ScriptDataDoubleEscapedState;
                        } else {
                            self.state = TokenizerState::ScriptDataEscapedState;
                        }
                        self.emit(Token::Character(c));
                    },
                    (Some(mut c @ 'A'..='Z'), _) => {
                        self.emit(Token::Character(c));
                        c.make_ascii_lowercase();
                        self.add_to_buffer(c);

                    },
                    (Some(c @ 'a'..='z'), _) => {
                        self.add_to_buffer(c);
                        self.emit(Token::Character(c));
                    },
                    _ => {
                        self.ptr -= 1;
                        self.state = TokenizerState::ScriptDataEscapedState;
                    },
                }
            },
            TokenizerState::ScriptDataDoubleEscapedState => {
                match self.read_next() {
                    Some('-') => {
                        self.state = TokenizerState::ScriptDataDoubleEscapedDashState;
                        self.emit(Token::Character('-'));
                    },
                    Some('<') => {
                        self.state = TokenizerState::ScriptDataDoubleEscapedLessThanSignState;
                        self.emit(Token::Character('<'));
                    },
                    Some('\0') => {
                        // unexpected null character parse error
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    },
                    Some(c) => {
                        self.emit(Token::Character(c));
                    },
                    None => {
                        // eof in script html comment like text parse error
                        self.emit(Token::EOF);
                    },
                }
            },
            TokenizerState::ScriptDataDoubleEscapedDashState => {
                match self.read_next() {
                    Some('-') => {
                        self.state = TokenizerState::ScriptDataDoubleEscapedDashDashState;
                        self.emit(Token::Character('-'));
                    },
                    Some('<') => {
                        self.state = TokenizerState::ScriptDataDoubleEscapedLessThanSignState;
                        self.emit(Token::Character('<'));
                    },
                    Some('\0') => {
                        // unexpected null character parse error
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    },
                    Some(c) => {
                        self.state = TokenizerState::ScriptDataDoubleEscapedState;
                        self.emit(Token::Character(c));
                    },
                    None => {
                        // eof in script html comment like text parse error
                        self.emit(Token::EOF);
                    },
                }
            },
            TokenizerState::ScriptDataDoubleEscapedDashDashState => {
                match self.read_next() {
                    Some('-') => {
                        self.emit(Token::Character('-'));
                    },
                    Some('<') => {
                        self.state = TokenizerState::ScriptDataDoubleEscapedLessThanSignState;
                        self.emit(Token::Character('<'));
                    },
                    Some('>') => {
                        self.state = TokenizerState::ScriptDataState;
                        self.emit(Token::Character('>'));
                    },
                    Some('\0') => {
                        // unexpected null character parse error
                        self.state = TokenizerState::ScriptDataDoubleEscapedState;
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    },
                    Some(c) => {
                        self.state = TokenizerState::ScriptDataDoubleEscapedState;
                        self.emit(Token::Character(c));
                    },
                    None => {
                        // eof in script html comment like text parse error
                        self.emit(Token::EOF);
                    },
                }
            },
            TokenizerState::ScriptDataDoubleEscapedLessThanSignState => {
                match self.read_next() {
                    Some('/') => {
                        self.buffer = Some(String::new());
                        self.state = TokenizerState::ScriptDataDoubleEscapeEndState;
                        self.emit(Token::Character('/'));
                    },
                    _ => {
                        self.ptr -= 1;
                        self.state = TokenizerState::ScriptDataDoubleEscapedState;
                    }
                },
            },
            TokenizerState::ScriptDataDoubleEscapeEndState {
                match self.read_next() {
                    //            tab       line feed    form feed     space
                    Some(c @ ('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' | '/' | '>')) => {
                        if self.buffer.contains(&"script") {
                            self.state = TokenizerState::ScriptDataEscapedState;
                        } else {
                            self.state = TokenizerState::ScriptDataDoubleEscapedState;
                        }
                        self.emit(Token::Character(c));
                    },
                    Some(mut c @ 'A'..='Z') => {
                        self.emit(Token::Character(c));
                        c.make_ascii_lowercase();
                        self.add_to_buffer(c);

                    },
                    Some(c @ 'a'..='z') => {
                        self.add_to_buffer(c);
                        self.emit(Token::Character(c));
                    },
                    _ => {
                        self.ptr -= 1;
                        self.state = TokenizerState::ScriptDataDoubleEscapedState;
                    }
            },
            TokenizerState::BeforeAttributeNameState => {
                match self.read_next() {
                    //      tab       line feed    form feed     space
                    Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}') => {}, // ignore
                    Some('/' | '>') | None => {
                        self.ptr -= 1;
                        self.state = TokenizerState::AfterAttributeNameState;
                    },
                    Some('=') => {
                        // unexpected equals sign before attribute name parse error
                        self.
                        self.state = TokenizerState::AttributeNameState;
                    }
                }
            },
            _ => unimplemented!("{:?}", self.state),
        }
    }
}
