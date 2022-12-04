use super::character_reference::match_reference;
use std::collections::VecDeque;

const UNICODE_REPLACEMENT: char = '\u{FFFD}';

#[derive(Debug, Clone, Copy)]
pub enum TokenizerState {
    DataState,
    RCDATAState,
    RAWTEXTState,
    ScriptDataState,
    // PLAINTEXTState,
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

#[derive(Debug, Clone)]
pub enum Token {
    DOCTYPE(Doctype),
    Tag(TagData),
    Comment(String),
    // TODO: emitting single characters is really inefficient, change this to be a string
    Character(char),
    EOF,
}

#[derive(Debug, Default, Clone)]
pub struct Doctype {
    pub name: Option<String>,
    pub public_ident: Option<String>,
    pub system_ident: Option<String>,
    pub force_quirks: bool,
}

#[derive(Debug, Clone)]
pub struct TagData {
    pub opening: bool,
    pub name: String,
    pub self_closing: bool,
    pub attributes: Vec<(String, String)>,
}

impl TagData {
    fn new_attribute(&mut self) {
        self.attributes.push((String::new(), String::new()));
    }

    /// Add a character to the last attribute's name
    fn add_to_attr_name(&mut self, c: char) {
        self.attributes.last_mut().unwrap().0.push(c);
    }

    /// Add a character to the last attribute's value
    fn add_to_attr_value(&mut self, c: char) {
        self.attributes.last_mut().unwrap().1.push(c);
    }

    fn default_open() -> Self {
        Self {
            opening: true,
            name: String::default(),
            self_closing: false,
            attributes: Vec::new(),
        }
    }

    fn default_close() -> Self {
        Self {
            opening: false,
            name: String::default(),
            self_closing: false,
            attributes: Vec::new(),
        }
    }
}

pub struct Tokenizer<'source> {
    source: &'source str,
    pub state: TokenizerState,
    ptr: usize,
    pub done: bool,
    token_buffer: VecDeque<Token>,

    /// Used by [TokenizerState::CharacterReferenceState]
    return_state: Option<TokenizerState>,
    last_emitted_start_tag_name: Option<String>,
    buffer: Option<String>,
    current_token: Option<Token>,
    character_reference_code: u32,
}

impl<'source> Tokenizer<'source> {
    pub fn new(source: &'source str) -> Self {
        Self {
            source: source,
            state: TokenizerState::DataState,
            return_state: None,
            current_token: None,
            last_emitted_start_tag_name: None,
            character_reference_code: 0,
            buffer: None,
            ptr: 0,
            done: false,
            token_buffer: VecDeque::new(),
        }
    }

    fn emit(&mut self, token: Token) {
        if let Token::Tag(TagData {
            opening: true,
            name,
            ..
        }) = &token
        {
            self.last_emitted_start_tag_name = Some(name.clone());
        }
        self.token_buffer.push_back(token);
    }

    fn emit_current_token(&mut self) {
        let current_token = self.current_token.take().unwrap();
        self.emit(current_token);
    }

    fn get_current_doctype(&mut self) -> &mut Doctype {
        match &mut self.current_token {
            Some(Token::DOCTYPE(ref mut d)) => d,
            _ => unreachable!(),
        }
    }

    fn get_current_tag(&mut self) -> &mut TagData {
        match &mut self.current_token {
            Some(Token::Tag(ref mut t)) => t,
            _ => unreachable!(),
        }
    }

    fn get_current_comment(&mut self) -> &mut String {
        match &mut self.current_token {
            Some(Token::Comment(s)) => s,
            _ => unreachable!(),
        }
    }

    /// Whether the current token is an [Token::EndTag] token whose name matches
    /// the name of the last [Token::StartTag] token that was emitted.
    fn is_appropriate_end_token(&self) -> bool {
        // Check if
        // * there was a start token emitted previously
        // * the token currently being emitted is an end token
        // * the name of the end token matches that of the start token
        match (&self.last_emitted_start_tag_name, &self.current_token) {
            (
                Some(open_name),
                Some(Token::Tag(TagData {
                    opening: false,
                    name: close_name,
                    ..
                })),
            ) => open_name == close_name,
            _ => false,
        }
    }

    fn add_to_buffer(&mut self, c: char) {
        match &mut self.buffer {
            Some(ref mut buffer) => {
                buffer.push(c);
            }
            None => panic!(
                "Trying to write {} to self.buffer but self.buffer is None",
                c
            ),
        }
    }

    /// Sets the current state to a specific state.
    /// All state transitions should call this method, which will
    /// ease debugging.
    pub fn switch_to(&mut self, state: TokenizerState) {
        self.state = state;
    }

    /// Reads the next character from the input strea,
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
                        self.switch_to(TokenizerState::CharacterReferenceState);
                    }
                    Some('<') => {
                        self.switch_to(TokenizerState::TagOpenState);
                    }
                    Some('\0') => {
                        // Unexpected null character parse error - emit replacement token
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    }
                    Some(c) => {
                        self.emit(Token::Character(c));
                    }
                    None => {
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::RCDATAState => {
                match self.read_next() {
                    Some('&') => {
                        self.return_state = Some(TokenizerState::RCDATAState);
                        self.switch_to(TokenizerState::CharacterReferenceState);
                    }
                    Some('<') => {
                        self.switch_to(TokenizerState::RCDATALessThanSignState);
                    }
                    Some('\0') => {
                        // Unexpected null character parse error - emit replacement token
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    }
                    Some(c) => {
                        self.emit(Token::Character(c));
                    }
                    None => {
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::RAWTEXTState => {
                match self.read_next() {
                    Some('<') => {
                        self.switch_to(TokenizerState::RAWTEXTLessThanSignState);
                    }
                    Some('\0') => {
                        // Unexpected null character parse error - emit replacement token
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    }
                    Some(c) => {
                        self.emit(Token::Character(c));
                    }
                    None => {
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::ScriptDataState => {
                match self.read_next() {
                    Some('<') => {
                        self.switch_to(TokenizerState::ScriptDataLessThanSignState);
                    }
                    Some('\0') => {
                        // Unexpected null character parse error - emit replacement token
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    }
                    Some(c) => {
                        self.emit(Token::Character(c));
                    }
                    None => {
                        self.emit(Token::EOF);
                    }
                }
            }
            // TokenizerState::PLAINTEXTState => {
            //     match self.read_next() {
            //         Some('\0') => {
            //             // Unexpected null character parse error - emit replacement token
            //             self.emit(Token::Character(UNICODE_REPLACEMENT));
            //         }
            //         Some(c) => {
            //             self.emit(Token::Character(c));
            //         }
            //         None => {
            //             self.emit(Token::EOF);
            //         }
            //     }
            // }
            TokenizerState::TagOpenState => {
                match self.read_next() {
                    Some('!') => {
                        self.switch_to(TokenizerState::MarkupDeclarationOpenState);
                    }
                    Some('/') => {
                        self.switch_to(TokenizerState::EndTagOpenState);
                    }
                    Some('a'..='z' | 'A'..='Z') => {
                        self.current_token = Some(Token::Tag(TagData::default_open()));
                        self.ptr -= 1; // reconsume the current character
                        self.switch_to(TokenizerState::TagNameState);
                    }
                    Some('?') => {
                        // Unexpected question mark instead of tag name parse error,
                        self.current_token = Some(Token::Comment(String::default()));
                        self.ptr -= 1; // reconsume current character
                        self.switch_to(TokenizerState::BogusCommentState);
                    }
                    Some(_) => {
                        // invalid-first-character-of-tag-name
                        self.ptr -= 1; // reconsume current token
                        self.switch_to(TokenizerState::DataState);
                        self.emit(Token::Character('<'));
                    }
                    None => {
                        // eof before tag name parse error
                        self.emit(Token::Character('<'));
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::EndTagOpenState => {
                match self.read_next() {
                    Some('a'..='z' | 'A'..='Z') => {
                        self.current_token = Some(Token::Tag(TagData::default_close()));
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::TagNameState);
                    }
                    Some('>') => {
                        // missing-end-tag-name parse error
                        self.switch_to(TokenizerState::DataState);
                    }
                    Some(_) => {
                        // invalid-first-character-of-tag-name parse error
                        self.current_token = Some(Token::Comment(String::default()));
                        self.ptr -= 1; // reconsume current character
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::BogusCommentState);
                    }
                    None => {
                        // eof-before-tag-name parse error
                        self.emit(Token::Character('<'));
                        self.emit(Token::Character('/'));
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::TagNameState => {
                match self.read_next() {
                    //       tab       line feed    form feed     space
                    Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}') => {
                        self.switch_to(TokenizerState::BeforeAttributeNameState);
                    }
                    Some('/') => {
                        self.switch_to(TokenizerState::SelfClosingStartTagState);
                    }
                    Some('>') => {
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    Some(mut c @ 'A'..='Z') => {
                        c.make_ascii_lowercase();
                        self.get_current_tag().name.push(c);
                    }
                    Some('\0') => {
                        // unexpected null character parse error
                        self.get_current_tag().name.push(UNICODE_REPLACEMENT);
                    }
                    Some(c) => {
                        self.get_current_tag().name.push(c);
                    }
                    None => {
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::RCDATALessThanSignState => match self.read_next() {
                Some('/') => {
                    self.buffer = Some(String::new());
                    self.switch_to(TokenizerState::RCDATAEndTagOpenState);
                }
                _ => {
                    self.emit(Token::Character('<'));
                    self.ptr -= 1;
                    self.switch_to(TokenizerState::RCDATAState);
                }
            },
            TokenizerState::RCDATAEndTagOpenState => match self.read_next() {
                Some('a'..='z' | 'A'..='Z') => {
                    self.current_token = Some(Token::Tag(TagData::default_close()));
                    self.ptr -= 1;
                    self.switch_to(TokenizerState::RCDATAEndTagNameState);
                }
                _ => {
                    self.emit(Token::Character('<'));
                    self.emit(Token::Character('/'));
                    self.ptr -= 1;
                    self.switch_to(TokenizerState::RCDATAState);
                }
            },
            TokenizerState::RCDATAEndTagNameState => {
                match (self.read_next(), self.is_appropriate_end_token()) {
                    //       tab       line feed    form feed     space
                    (Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}'), true) => {
                        self.switch_to(TokenizerState::BeforeAttributeNameState);
                    }
                    (Some('/'), true) => {
                        self.switch_to(TokenizerState::SelfClosingStartTagState);
                    }
                    (Some('>'), true) => {
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    (Some(mut c @ 'A'..='Z'), _) => {
                        self.add_to_buffer(c);
                        c.make_ascii_lowercase();
                        self.get_current_tag().name.push(c);
                    }
                    (Some(c @ 'a'..='z'), _) => {
                        self.add_to_buffer(c);
                        self.get_current_tag().name.push(c);
                    }
                    _ => {
                        self.emit(Token::Character('<'));
                        self.emit(Token::Character('/'));
                        for c in self.buffer.take().unwrap().chars() {
                            self.emit(Token::Character(c));
                        }
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::RCDATAState);
                    }
                }
            }
            TokenizerState::RAWTEXTLessThanSignState => match self.read_next() {
                Some('/') => {
                    self.buffer = Some(String::new());
                    self.switch_to(TokenizerState::RAWTEXTEndTagOpenState);
                }
                _ => {
                    self.emit(Token::Character('<'));
                    self.ptr -= 1;
                    self.switch_to(TokenizerState::RAWTEXTState);
                }
            },
            TokenizerState::RAWTEXTEndTagOpenState => match self.read_next() {
                Some('a'..='z' | 'A'..='Z') => {
                    self.current_token = Some(Token::Tag(TagData::default_close()));
                    self.ptr -= 1;
                    self.switch_to(TokenizerState::RAWTEXTEndTagNameState);
                }
                _ => {
                    self.emit(Token::Character('<'));
                    self.emit(Token::Character('/'));
                    self.ptr -= 1;
                    self.switch_to(TokenizerState::RAWTEXTState);
                }
            },
            TokenizerState::RAWTEXTEndTagNameState => {
                match (self.read_next(), self.is_appropriate_end_token()) {
                    //       tab       line feed    form feed     space
                    (Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}'), true) => {
                        self.switch_to(TokenizerState::BeforeAttributeNameState);
                    }
                    (Some('/'), true) => {
                        self.switch_to(TokenizerState::SelfClosingStartTagState);
                    }
                    (Some('>'), true) => {
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    (Some(mut c @ 'A'..='Z'), _) => {
                        self.add_to_buffer(c);
                        c.make_ascii_lowercase();
                        self.get_current_tag().name.push(c);
                    }
                    (Some(c @ 'a'..='z'), _) => {
                        self.add_to_buffer(c);
                        self.get_current_tag().name.push(c);
                    }
                    _ => {
                        self.emit(Token::Character('<'));
                        self.emit(Token::Character('/'));
                        for c in self.buffer.take().unwrap().chars() {
                            self.emit(Token::Character(c));
                        }
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::RAWTEXTState);
                    }
                }
            }
            TokenizerState::ScriptDataLessThanSignState => match self.read_next() {
                Some('/') => {
                    self.buffer = Some(String::new());
                    self.switch_to(TokenizerState::ScriptDataEndTagOpenState);
                }
                Some('!') => {
                    self.switch_to(TokenizerState::ScriptDataEscapeStartState);
                    self.emit(Token::Character('<'));
                    self.emit(Token::Character('!'));
                }
                _ => {
                    self.emit(Token::Character('<'));
                    self.ptr -= 1;
                    self.switch_to(TokenizerState::ScriptDataState);
                }
            },
            TokenizerState::ScriptDataEndTagOpenState => match self.read_next() {
                Some('a'..='z' | 'A'..='Z') => {
                    self.current_token = Some(Token::Tag(TagData::default_close()));
                    self.ptr -= 1;
                    self.switch_to(TokenizerState::ScriptDataEndTagNameState);
                }
                _ => {
                    self.emit(Token::Character('<'));
                    self.emit(Token::Character('/'));
                    self.ptr -= 1;
                    self.switch_to(TokenizerState::ScriptDataState);
                }
            },
            TokenizerState::ScriptDataEndTagNameState => {
                match (self.read_next(), self.is_appropriate_end_token()) {
                    //       tab       line feed    form feed     space
                    (Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}'), true) => {
                        self.switch_to(TokenizerState::BeforeAttributeNameState);
                    }
                    (Some('/'), true) => {
                        self.switch_to(TokenizerState::SelfClosingStartTagState);
                    }
                    (Some('>'), true) => {
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    (Some(mut c @ 'A'..='Z'), _) => {
                        self.add_to_buffer(c);
                        c.make_ascii_lowercase();
                        self.get_current_tag().name.push(c);
                    }
                    (Some(c @ 'a'..='z'), _) => {
                        self.add_to_buffer(c);
                        self.get_current_tag().name.push(c);
                    }
                    _ => {
                        self.emit(Token::Character('<'));
                        self.emit(Token::Character('/'));
                        for c in self.buffer.take().unwrap().chars() {
                            self.emit(Token::Character(c));
                        }
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::ScriptDataState);
                    }
                }
            }
            TokenizerState::ScriptDataEscapeStartState => match self.read_next() {
                Some('-') => {
                    self.switch_to(TokenizerState::ScriptDataEscapeStartDashState);
                    self.emit(Token::Character('-'));
                }
                _ => {
                    self.ptr -= 1;
                    self.switch_to(TokenizerState::ScriptDataState);
                }
            },
            TokenizerState::ScriptDataEscapeStartDashState => match self.read_next() {
                Some('-') => {
                    self.switch_to(TokenizerState::ScriptDataEscapedDashDashState);
                    self.emit(Token::Character('-'));
                }
                _ => {
                    self.ptr -= 1;
                    self.switch_to(TokenizerState::ScriptDataState);
                }
            },
            TokenizerState::ScriptDataEscapedState => {
                match self.read_next() {
                    Some('-') => {
                        self.switch_to(TokenizerState::ScriptDataEscapedDashState);
                        self.emit(Token::Character('-'));
                    }
                    Some('<') => {
                        self.switch_to(TokenizerState::ScriptDataEscapedLessThanSignState);
                    }
                    Some('\0') => {
                        // unexpected null character parse error
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    }
                    Some(c) => {
                        self.emit(Token::Character(c));
                    }
                    None => {
                        // eof in script html comment like text parse error
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::ScriptDataEscapedDashState => {
                match self.read_next() {
                    Some('-') => {
                        self.switch_to(TokenizerState::ScriptDataEscapedDashDashState);
                        self.emit(Token::Character('-'));
                    }
                    Some('<') => {
                        self.switch_to(TokenizerState::ScriptDataEscapedLessThanSignState);
                    }
                    Some('\0') => {
                        // unexpected null character parse error
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    }
                    Some(c) => {
                        self.switch_to(TokenizerState::ScriptDataEscapedState);
                        self.emit(Token::Character(c));
                    }
                    None => {
                        // eof in script html comment like text parse error
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::ScriptDataEscapedDashDashState => {
                match self.read_next() {
                    Some('-') => {
                        self.emit(Token::Character('-'));
                    }
                    Some('<') => {
                        self.switch_to(TokenizerState::ScriptDataEscapedLessThanSignState);
                    }
                    Some('>') => {
                        self.switch_to(TokenizerState::ScriptDataState);
                        self.emit(Token::Character('>'));
                    }
                    Some('\0') => {
                        // unexpected null character parse error
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    }
                    Some(c) => {
                        self.switch_to(TokenizerState::ScriptDataEscapedState);
                        self.emit(Token::Character(c));
                    }
                    None => {
                        // eof in script html comment like text parse error
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::ScriptDataEscapedLessThanSignState => match self.read_next() {
                Some('/') => {
                    self.buffer = Some(String::new());
                    self.switch_to(TokenizerState::ScriptDataEscapedEndTagOpenState);
                }
                Some('a'..='z' | 'A'..='Z') => {
                    self.buffer = Some(String::new());
                    self.emit(Token::Character('<'));
                    self.ptr -= 1;
                    self.switch_to(TokenizerState::ScriptDataDoubleEscapeStartState);
                }
                _ => {
                    self.emit(Token::Character('<'));
                    self.ptr -= 1;
                    self.switch_to(TokenizerState::ScriptDataEscapedState);
                }
            },
            TokenizerState::ScriptDataEscapedEndTagOpenState => match self.read_next() {
                Some('a'..='z' | 'A'..='Z') => {
                    self.current_token = Some(Token::Tag(TagData::default_close()));
                    self.ptr -= 1;
                    self.switch_to(TokenizerState::ScriptDataEscapedEndTagNameState);
                }
                _ => {
                    self.emit(Token::Character('<'));
                    self.emit(Token::Character('/'));
                    self.ptr -= 1;
                    self.switch_to(TokenizerState::ScriptDataEscapedState);
                }
            },
            TokenizerState::ScriptDataEscapedEndTagNameState => {
                match (self.read_next(), self.is_appropriate_end_token()) {
                    //       tab       line feed    form feed     space
                    (Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}'), true) => {
                        self.switch_to(TokenizerState::BeforeAttributeNameState);
                    }
                    (Some('/'), true) => {
                        self.switch_to(TokenizerState::SelfClosingStartTagState);
                    }
                    (Some('>'), true) => {
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    (Some(mut c @ 'A'..='Z'), _) => {
                        self.add_to_buffer(c);
                        c.make_ascii_lowercase();
                        self.get_current_tag().name.push(c);
                    }
                    (Some(c @ 'a'..='z'), _) => {
                        self.add_to_buffer(c);
                        self.get_current_tag().name.push(c);
                    }
                    _ => {
                        self.emit(Token::Character('<'));
                        self.emit(Token::Character('/'));
                        for c in self.buffer.take().unwrap().chars() {
                            self.emit(Token::Character(c));
                        }
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::ScriptDataEscapedState);
                    }
                }
            }
            TokenizerState::ScriptDataDoubleEscapeStartState => {
                match self.read_next() {
                    //             tab       line feed    form feed     space
                    Some(c @ ('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' | '/' | '>')) => {
                        if self.buffer.contains(&"script") {
                            self.switch_to(TokenizerState::ScriptDataDoubleEscapedState);
                        } else {
                            self.switch_to(TokenizerState::ScriptDataEscapedState);
                        }
                        self.emit(Token::Character(c));
                    }
                    Some(mut c @ 'A'..='Z') => {
                        self.emit(Token::Character(c));
                        c.make_ascii_lowercase();
                        self.add_to_buffer(c);
                    }
                    Some(c @ 'a'..='z') => {
                        self.add_to_buffer(c);
                        self.emit(Token::Character(c));
                    }
                    _ => {
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::ScriptDataEscapedState);
                    }
                }
            }
            TokenizerState::ScriptDataDoubleEscapedState => {
                match self.read_next() {
                    Some('-') => {
                        self.switch_to(TokenizerState::ScriptDataDoubleEscapedDashState);
                        self.emit(Token::Character('-'));
                    }
                    Some('<') => {
                        self.switch_to(TokenizerState::ScriptDataDoubleEscapedLessThanSignState);
                        self.emit(Token::Character('<'));
                    }
                    Some('\0') => {
                        // unexpected null character parse error
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    }
                    Some(c) => {
                        self.emit(Token::Character(c));
                    }
                    None => {
                        // eof in script html comment like text parse error
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::ScriptDataDoubleEscapedDashState => {
                match self.read_next() {
                    Some('-') => {
                        self.switch_to(TokenizerState::ScriptDataDoubleEscapedDashDashState);
                        self.emit(Token::Character('-'));
                    }
                    Some('<') => {
                        self.switch_to(TokenizerState::ScriptDataDoubleEscapedLessThanSignState);
                        self.emit(Token::Character('<'));
                    }
                    Some('\0') => {
                        // unexpected null character parse error
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    }
                    Some(c) => {
                        self.switch_to(TokenizerState::ScriptDataDoubleEscapedState);
                        self.emit(Token::Character(c));
                    }
                    None => {
                        // eof in script html comment like text parse error
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::ScriptDataDoubleEscapedDashDashState => {
                match self.read_next() {
                    Some('-') => {
                        self.emit(Token::Character('-'));
                    }
                    Some('<') => {
                        self.switch_to(TokenizerState::ScriptDataDoubleEscapedLessThanSignState);
                        self.emit(Token::Character('<'));
                    }
                    Some('>') => {
                        self.switch_to(TokenizerState::ScriptDataState);
                        self.emit(Token::Character('>'));
                    }
                    Some('\0') => {
                        // unexpected null character parse error
                        self.switch_to(TokenizerState::ScriptDataDoubleEscapedState);
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    }
                    Some(c) => {
                        self.switch_to(TokenizerState::ScriptDataDoubleEscapedState);
                        self.emit(Token::Character(c));
                    }
                    None => {
                        // eof in script html comment like text parse error
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::ScriptDataDoubleEscapedLessThanSignState => match self.read_next() {
                Some('/') => {
                    self.buffer = Some(String::new());
                    self.switch_to(TokenizerState::ScriptDataDoubleEscapeEndState);
                    self.emit(Token::Character('/'));
                }
                _ => {
                    self.ptr -= 1;
                    self.switch_to(TokenizerState::ScriptDataDoubleEscapedState);
                }
            },
            TokenizerState::ScriptDataDoubleEscapeEndState => {
                match self.read_next() {
                    //            tab       line feed    form feed     space
                    Some(c @ ('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' | '/' | '>')) => {
                        if self.buffer.contains(&"script") {
                            self.switch_to(TokenizerState::ScriptDataEscapedState);
                        } else {
                            self.switch_to(TokenizerState::ScriptDataDoubleEscapedState);
                        }
                        self.emit(Token::Character(c));
                    }
                    Some(mut c @ 'A'..='Z') => {
                        self.emit(Token::Character(c));
                        c.make_ascii_lowercase();
                        self.add_to_buffer(c);
                    }
                    Some(c @ 'a'..='z') => {
                        self.add_to_buffer(c);
                        self.emit(Token::Character(c));
                    }
                    _ => {
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::ScriptDataDoubleEscapedState);
                    }
                }
            }
            TokenizerState::BeforeAttributeNameState => {
                match self.read_next() {
                    //      tab       line feed    form feed     space
                    Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}') => {} // ignore
                    Some('/' | '>') | None => {
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::AfterAttributeNameState);
                    }
                    Some('=') => {
                        // unexpected equals sign before attribute name parse error
                        self.get_current_tag().new_attribute();
                        self.get_current_tag().add_to_attr_name('=');
                        self.switch_to(TokenizerState::AttributeNameState);
                    }
                    _ => {
                        self.get_current_tag().new_attribute();
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::AttributeNameState);
                    }
                }
            }
            TokenizerState::AttributeNameState => {
                // TODO: when leaving the AttributeNameState, we need to check
                // for duplicate attribute names.
                // https://html.spec.whatwg.org/multipage/parsing.html#attribute-name-state
                match self.read_next() {
                    //      tab       line feed    form feed     space
                    Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' | '/' | '>') | None => {
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::AfterAttributeNameState);
                    }
                    Some('=') => {
                        self.switch_to(TokenizerState::BeforeAttributeValueState);
                    }
                    Some(mut c @ 'A'..='Z') => {
                        c.make_ascii_lowercase();
                        self.get_current_tag().add_to_attr_name(c);
                    }
                    Some('\0') => {
                        // unexpected null character parse error
                        self.get_current_tag().add_to_attr_name(UNICODE_REPLACEMENT);
                    }
                    Some(c @ ('"' | '\'' | '<')) => {
                        // unexpected character in attribute name parse error
                        self.get_current_tag().add_to_attr_name(c);
                    }
                    Some(c) => {
                        self.get_current_tag().add_to_attr_name(c);
                    }
                }
            }
            TokenizerState::AfterAttributeNameState => {
                match self.read_next() {
                    //      tab       line feed    form feed     space
                    Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}') => {} // ignore
                    Some('/') => {
                        self.switch_to(TokenizerState::SelfClosingStartTagState);
                    }
                    Some('=') => {
                        self.switch_to(TokenizerState::BeforeAttributeValueState);
                    }
                    Some('>') => {
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    Some(_) => {
                        self.get_current_tag().new_attribute();
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::AttributeNameState);
                    }
                    None => {
                        // eof in tag parse error
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::BeforeAttributeValueState => {
                match self.read_next() {
                    //      tab       line feed    form feed     space
                    Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}') => {} // ignore
                    Some('"') => {
                        self.switch_to(TokenizerState::AttributeValueDoublequotedState);
                    }
                    Some('\'') => {
                        self.switch_to(TokenizerState::AttributeValueSinglequotedState);
                    }
                    Some('>') => {
                        // missing attribute value parse error
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    _ => {
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::AttributeValueUnquotedState);
                    }
                }
            }
            TokenizerState::AttributeValueDoublequotedState => {
                match self.read_next() {
                    Some('"') => {
                        self.switch_to(TokenizerState::AfterAttributeValueQuotedState);
                    }
                    Some('&') => {
                        self.return_state = Some(TokenizerState::AttributeValueDoublequotedState);
                        self.switch_to(TokenizerState::CharacterReferenceState);
                    }
                    Some('\0') => {
                        // unexpected null character parse error
                        self.get_current_tag()
                            .add_to_attr_value(UNICODE_REPLACEMENT);
                    }
                    Some(c) => {
                        self.get_current_tag().add_to_attr_value(c);
                    }
                    None => {
                        // eof in tag parse error
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::AttributeValueSinglequotedState => {
                match self.read_next() {
                    Some('\'') => {
                        self.switch_to(TokenizerState::AfterAttributeValueQuotedState);
                    }
                    Some('&') => {
                        self.return_state = Some(TokenizerState::AttributeValueSinglequotedState);
                        self.switch_to(TokenizerState::CharacterReferenceState);
                    }
                    Some('\0') => {
                        // unexpected null character parse error
                        self.get_current_tag()
                            .add_to_attr_value(UNICODE_REPLACEMENT);
                    }
                    Some(c) => {
                        self.get_current_tag().add_to_attr_value(c);
                    }
                    None => {
                        // eof in tag parse error
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::AttributeValueUnquotedState => {
                match self.read_next() {
                    //      tab       line feed    form feed     space
                    Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}') => {
                        self.switch_to(TokenizerState::BeforeAttributeNameState);
                    }
                    Some('&') => {
                        self.return_state = Some(TokenizerState::AttributeValueUnquotedState);
                        self.switch_to(TokenizerState::CharacterReferenceState);
                    }
                    Some('>') => {
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    Some('\0') => {
                        // unexpected null character parse error
                        self.get_current_tag()
                            .add_to_attr_value(UNICODE_REPLACEMENT);
                    }
                    Some(c @ ('"' | '\'' | '<' | '=' | '`')) => {
                        // unexpected character in unquoted attribute value parse error
                        self.get_current_tag().add_to_attr_value(c);
                    }
                    Some(c) => {
                        self.get_current_tag().add_to_attr_value(c);
                    }
                    None => {
                        // eof in tag parse error
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::AfterAttributeValueQuotedState => {
                match self.read_next() {
                    //      tab       line feed    form feed     space
                    Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}') => {
                        self.switch_to(TokenizerState::BeforeAttributeNameState);
                    }
                    Some('/') => {
                        self.switch_to(TokenizerState::SelfClosingStartTagState);
                    }
                    Some('>') => {
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    Some(_) => {
                        // missing whitespace between attributes parse error
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::BeforeAttributeNameState);
                    }
                    None => {
                        // eof in tag parse error
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::SelfClosingStartTagState => {
                match self.read_next() {
                    Some('>') => {
                        self.switch_to(TokenizerState::DataState);
                        self.get_current_tag().self_closing = true;
                        self.emit_current_token();
                    }
                    Some(_) => {
                        // unexpected solidus in tag parse error
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::BeforeAttributeNameState);
                    }
                    None => {
                        // eof in tag parse error
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::BogusCommentState => {
                match self.read_next() {
                    Some('>') => {
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    Some('\0') => {
                        // unexpected null character parse error
                        self.get_current_comment().push(UNICODE_REPLACEMENT);
                    }
                    Some(c) => {
                        self.get_current_comment().push(c);
                    }
                    None => {
                        self.emit_current_token();
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::MarkupDeclarationOpenState => {
                if &self.source[self.ptr..self.ptr + 2] == "--" {
                    self.ptr += 2;
                    self.current_token = Some(Token::Comment(String::default()));
                    self.switch_to(TokenizerState::CommentStartState);
                } else if self.source[self.ptr..self.ptr + 7].eq_ignore_ascii_case("DOCTYPE") {
                    self.ptr += 7;
                    self.switch_to(TokenizerState::DOCTYPEState);
                } else if &self.source[self.ptr..self.ptr + 7] == "[CDATA[" {
                    self.ptr += 7;
                    todo!();
                } else {
                    // incorrectly opened comment parse error
                    self.current_token = Some(Token::Comment(String::default()));
                    self.switch_to(TokenizerState::BogusCommentState);
                }
            }
            TokenizerState::CommentStartState => {
                match self.read_next() {
                    Some('-') => {
                        self.switch_to(TokenizerState::CommentStartDashState);
                    }
                    Some('>') => {
                        // abrupt closing of empty comment parse error
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    _ => {
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::CommentState);
                    }
                }
            }
            TokenizerState::CommentStartDashState => {
                match self.read_next() {
                    Some('-') => {
                        self.switch_to(TokenizerState::CommentEndState);
                    }
                    Some('>') => {
                        // abrupt closing of empty comment parse error
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    Some(_) => {
                        self.get_current_comment().push('-');
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::CommentState);
                    }
                    None => {
                        // eof in comment parse error
                        self.emit_current_token();
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::CommentState => {
                match self.read_next() {
                    Some('>') => {
                        self.get_current_comment().push('<');
                        self.switch_to(TokenizerState::CommentLessThanSignState);
                    }
                    Some('-') => {
                        self.switch_to(TokenizerState::CommentEndDashState);
                    }
                    Some('\0') => {
                        // unexpected null character parse error
                        self.get_current_comment().push(UNICODE_REPLACEMENT);
                    }
                    Some(c) => {
                        self.get_current_comment().push(c);
                    }
                    None => {
                        // eof in comment parse error
                        self.emit_current_token();
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::CommentLessThanSignState => match self.read_next() {
                Some('!') => {
                    self.get_current_comment().push('!');
                    self.switch_to(TokenizerState::CommentLessThanSignBangState);
                }
                Some('<') => {
                    self.get_current_comment().push('<');
                }
                _ => {
                    self.ptr -= 1;
                    self.switch_to(TokenizerState::CommentState);
                }
            },
            TokenizerState::CommentLessThanSignBangState => match self.read_next() {
                Some('-') => {
                    self.switch_to(TokenizerState::CommentLessThanSignBangDashState);
                }
                _ => {
                    self.ptr -= 1;
                    self.switch_to(TokenizerState::CommentState);
                }
            },
            TokenizerState::CommentLessThanSignBangDashState => match self.read_next() {
                Some('-') => {
                    self.switch_to(TokenizerState::CommentLessThanSignBangDashDashState);
                }
                _ => {
                    self.ptr -= 1;
                    self.switch_to(TokenizerState::CommentEndDashState);
                }
            },
            TokenizerState::CommentLessThanSignBangDashDashState => {
                match self.read_next() {
                    Some('>') | None => {
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::CommentEndState);
                    }
                    Some(_) => {
                        // nested comment parse error
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::CommentEndState);
                    }
                }
            }
            TokenizerState::CommentEndDashState => {
                match self.read_next() {
                    Some('-') => {
                        self.switch_to(TokenizerState::CommentEndState);
                    }
                    Some(_) => {
                        self.get_current_comment().push('-');
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::CommentState);
                    }
                    None => {
                        // eof in comment parse error
                        self.emit_current_token();
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::CommentEndState => {
                match self.read_next() {
                    Some('>') => {
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    Some('!') => {
                        self.switch_to(TokenizerState::CommentEndBangState);
                    }
                    Some('-') => {
                        self.get_current_comment().push('-');
                    }
                    Some(_) => {
                        self.get_current_comment().push_str("--");
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::CommentState);
                    }
                    None => {
                        // eof in comment parse error
                        self.emit_current_token();
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::CommentEndBangState => {
                match self.read_next() {
                    Some('-') => {
                        self.get_current_comment().push_str("--!");
                        self.switch_to(TokenizerState::CommentEndDashState);
                    }
                    Some('>') => {
                        // incorrectly closed comment parse error
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    Some(_) => {
                        self.get_current_comment().push_str("--!");
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::CommentState);
                    }
                    None => {
                        // eof in comment parse error
                        self.emit_current_token();
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::DOCTYPEState => {
                match self.read_next() {
                    //       tab       line feed    form feed     space
                    Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}') => {
                        self.switch_to(TokenizerState::BeforeDOCTYPENameState);
                    }
                    Some('>') => {
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::BeforeDOCTYPENameState);
                    }
                    Some(_) => {
                        // missing whitespace before doctype name parse error
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::BeforeDOCTYPENameState);
                    }
                    None => {
                        // eof in doctype parse error
                        self.current_token = Some(Token::DOCTYPE(Doctype::default()));
                        self.get_current_doctype().force_quirks = true;
                        self.emit_current_token();
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::BeforeDOCTYPENameState => {
                // Note: this code potentially emits tokens *without* modifying self.current_token!
                let mut doctype_token = Doctype::default();

                match self.read_next() {
                    //       tab       line feed    form feed     space
                    Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}') => {}
                    Some(c @ 'A'..='Z') => {
                        doctype_token.name = Some(c.to_ascii_lowercase().to_string());
                        self.current_token = Some(Token::DOCTYPE(doctype_token));
                        self.switch_to(TokenizerState::DOCTYPENameState);
                    }
                    Some('\0') => {
                        // unexpected null character parse error
                        doctype_token.name = Some(UNICODE_REPLACEMENT.to_string());
                        self.current_token = Some(Token::DOCTYPE(doctype_token));
                        self.switch_to(TokenizerState::DOCTYPENameState);
                    }
                    Some('>') => {
                        // missing doctype name parse error
                        doctype_token.force_quirks = true;
                        self.emit(Token::DOCTYPE(doctype_token));
                        self.switch_to(TokenizerState::DataState);
                    }
                    Some(c) => {
                        doctype_token.name = Some(c.to_string());
                        self.current_token = Some(Token::DOCTYPE(doctype_token));
                        self.switch_to(TokenizerState::DOCTYPENameState);
                    }
                    None => {
                        // eof in doctype parse error
                        doctype_token.force_quirks = true;
                        self.emit(Token::DOCTYPE(doctype_token));
                        self.emit(Token::EOF)
                    }
                }
            }
            TokenizerState::DOCTYPENameState => {
                match self.read_next() {
                    //       tab       line feed    form feed     space
                    Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}') => {
                        self.switch_to(TokenizerState::AfterDOCTYPENameState);
                    }
                    Some('>') => {
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    Some(c @ 'A'..='Z') => {
                        self.get_current_doctype()
                            .name
                            .as_mut()
                            .map(|name| name.push(c.to_ascii_lowercase()));
                    }
                    Some('\0') => {
                        // unexpected null character parse error
                        self.get_current_doctype()
                            .name
                            .as_mut()
                            .map(|name| name.push(UNICODE_REPLACEMENT));
                    }
                    Some(c) => {
                        self.get_current_doctype()
                            .name
                            .as_mut()
                            .map(|name| name.push(c));
                    }
                    None => {
                        // eof in doctype parse error
                        self.get_current_doctype().force_quirks = true;
                        self.emit_current_token();
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::AfterDOCTYPENameState => {
                match self.read_next() {
                    //       tab       line feed    form feed     space
                    Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}') => {}
                    Some('>') => {
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    None => {
                        // eof in doctype parse error
                        self.get_current_doctype().force_quirks = true;
                        self.emit_current_token();
                        self.emit(Token::EOF);
                    }
                    Some(_) => {
                        self.ptr -= 1;
                        let next_six_chars = &self.source[self.ptr..self.ptr + 6];
                        if next_six_chars.eq_ignore_ascii_case("PUBLIC") {
                            self.ptr += 6;
                            self.switch_to(TokenizerState::AfterDOCTYPEPublicKeywordState);
                        } else if next_six_chars.eq_ignore_ascii_case("SYSTEM") {
                            self.ptr += 6;
                            self.switch_to(TokenizerState::AfterDOCTYPESystemKeywordState);
                        } else {
                            // invalid character sequence after doctype name parse error
                            self.get_current_doctype().force_quirks = true;
                            // Note: we reconsume, but because we already decremented
                            // self.ptr (above) we don't need to do it again
                            self.switch_to(TokenizerState::BogusDOCTYPEState);
                        }
                    }
                }
            }
            TokenizerState::AfterDOCTYPEPublicKeywordState => {
                match self.read_next() {
                    //       tab       line feed    form feed     space
                    Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}') => {
                        self.switch_to(TokenizerState::BeforeDOCTYPEPublicIdentifierState);
                    }
                    Some('"') => {
                        // missing whitespace after doctype public keyword parse error
                        self.get_current_doctype().public_ident = Some(String::new());
                        self.switch_to(TokenizerState::DOCTYPEPublicIdentifierDoublequotedState);
                    }
                    Some('\'') => {
                        // missing whitespace after doctype public keyword parse error
                        self.get_current_doctype().public_ident = Some(String::new());
                        self.switch_to(TokenizerState::DOCTYPEPublicIdentifierSinglequotedState);
                    }
                    Some('>') => {
                        // missing doctype public identifier parse error
                        self.get_current_doctype().force_quirks = true;
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    Some(_) => {
                        // missing quote before doctype public identifier parse error
                        self.get_current_doctype().force_quirks = true;
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::BogusDOCTYPEState);
                    }
                    None => {
                        // eof in doctype parse error
                        self.get_current_doctype().force_quirks = true;
                        self.emit_current_token();
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::BeforeDOCTYPEPublicIdentifierState => {
                match self.read_next() {
                    //       tab       line feed    form feed     space
                    Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}') => {}
                    Some('"') => {
                        self.get_current_doctype().public_ident = Some(String::new());
                        self.switch_to(TokenizerState::DOCTYPEPublicIdentifierDoublequotedState);
                    }
                    Some('\'') => {
                        self.get_current_doctype().public_ident = Some(String::new());
                        self.switch_to(TokenizerState::DOCTYPEPublicIdentifierSinglequotedState);
                    }
                    Some('>') => {
                        // missing doctype public identifier parse error
                        self.get_current_doctype().force_quirks = true;
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    Some(_) => {
                        // missing quote before doctype public identifier parse error
                        self.get_current_doctype().force_quirks = true;
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::BogusDOCTYPEState);
                    }
                    None => {
                        // eof in doctype parse error
                        self.get_current_doctype().force_quirks = true;
                        self.emit_current_token();
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::DOCTYPEPublicIdentifierDoublequotedState => {
                match self.read_next() {
                    Some('"') => {
                        self.switch_to(TokenizerState::AfterDOCTYPEPublicIdentifierState);
                    }
                    Some('\0') => {
                        // unexpected null character parse error
                        self.get_current_doctype()
                            .public_ident
                            .as_mut()
                            .map(|ident| ident.push(UNICODE_REPLACEMENT));
                    }
                    Some('>') => {
                        // abrupt doctype public identifier parse error
                        self.get_current_doctype().force_quirks = true;
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    Some(c) => {
                        self.get_current_doctype()
                            .public_ident
                            .as_mut()
                            .map(|ident| ident.push(c));
                    }
                    None => {
                        // eof in doctype parse error
                        self.get_current_doctype().force_quirks = true;
                        self.emit_current_token();
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::DOCTYPEPublicIdentifierSinglequotedState => {
                match self.read_next() {
                    Some('\'') => {
                        self.switch_to(TokenizerState::AfterDOCTYPEPublicIdentifierState);
                    }
                    Some('\0') => {
                        // unexpected null character parse error
                        self.get_current_doctype()
                            .public_ident
                            .as_mut()
                            .map(|ident| ident.push(UNICODE_REPLACEMENT));
                    }
                    Some('>') => {
                        // abrupt doctype public identifier parse error
                        self.get_current_doctype().force_quirks = true;
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    Some(c) => {
                        self.get_current_doctype()
                            .public_ident
                            .as_mut()
                            .map(|ident| ident.push(c));
                    }
                    None => {
                        // eof in doctype parse error
                        self.get_current_doctype().force_quirks = true;
                        self.emit_current_token();
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::AfterDOCTYPEPublicIdentifierState => {
                match self.read_next() {
                    //       tab       line feed    form feed     space
                    Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}') => {
                        self.switch_to(
                            TokenizerState::BetweenDOCTYPEPublicAndSystemIdentifiersState,
                        );
                    }
                    Some('>') => {
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    Some('"') => {
                        // missing whitespace between doctype public and system identifiers parse error
                        self.get_current_doctype().system_ident = Some(String::new());
                        self.switch_to(TokenizerState::DOCTYPESystemIdentifierDoublequotedState);
                    }
                    Some('\'') => {
                        // missing whitespace between doctype public and system identifiers parse error
                        self.get_current_doctype().system_ident = Some(String::new());
                        self.switch_to(TokenizerState::DOCTYPESystemIdentifierSinglequotedState);
                    }
                    Some(_) => {
                        // missing quote before doctype system identifier parse error
                        self.get_current_doctype().force_quirks = true;
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::BogusDOCTYPEState);
                    }
                    None => {
                        // eof in doctype parse error
                        self.get_current_doctype().force_quirks = true;
                        self.emit_current_token();
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::BetweenDOCTYPEPublicAndSystemIdentifiersState => {
                match self.read_next() {
                    //       tab       line feed    form feed     space
                    Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}') => {}
                    Some('>') => {
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    Some('"') => {
                        self.get_current_doctype().system_ident = Some(String::new());
                        self.switch_to(TokenizerState::DOCTYPESystemIdentifierDoublequotedState);
                    }
                    Some('\'') => {
                        self.get_current_doctype().system_ident = Some(String::new());
                        self.switch_to(TokenizerState::DOCTYPESystemIdentifierSinglequotedState);
                    }
                    Some(_) => {
                        // missing quote before doctype system identifier parse error
                        self.get_current_doctype().force_quirks = true;
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::BogusDOCTYPEState);
                    }
                    None => {
                        // eof in doctype parse error
                        self.get_current_doctype().force_quirks = true;
                        self.emit_current_token();
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::AfterDOCTYPESystemKeywordState => {
                match self.read_next() {
                    //       tab       line feed    form feed     space
                    Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}') => {
                        self.switch_to(TokenizerState::BeforeDOCTYPESystemIdentifierState);
                    }
                    Some('"') => {
                        // missing whitespace after doctype system keyword parse error
                        self.get_current_doctype().system_ident = Some(String::new());
                        self.switch_to(TokenizerState::DOCTYPESystemIdentifierDoublequotedState);
                    }
                    Some('\'') => {
                        // missing whitespace after doctype system keyword parse error
                        self.get_current_doctype().system_ident = Some(String::new());
                        self.switch_to(TokenizerState::DOCTYPESystemIdentifierSinglequotedState);
                    }
                    Some('>') => {
                        // missing doctype system identifier parse error
                        self.get_current_doctype().force_quirks = true;
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    Some(_) => {
                        // missing quote before doctype system identifier parse error
                        self.get_current_doctype().force_quirks = true;
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::BogusDOCTYPEState);
                    }
                    None => {
                        self.get_current_doctype().force_quirks = true;
                        self.emit_current_token();
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::BeforeDOCTYPESystemIdentifierState => {
                match self.read_next() {
                    //       tab       line feed    form feed     space
                    Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}') => {}
                    Some('"') => {
                        self.get_current_doctype().system_ident = Some(String::new());
                        self.switch_to(TokenizerState::DOCTYPESystemIdentifierDoublequotedState);
                    }
                    Some('\'') => {
                        self.get_current_doctype().system_ident = Some(String::new());
                        self.switch_to(TokenizerState::DOCTYPESystemIdentifierSinglequotedState);
                    }
                    Some('>') => {
                        // missing doctype system identifier parse error
                        self.get_current_doctype().force_quirks = true;
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    Some(_) => {
                        // missing quote before doctype system identifier parse error
                        self.get_current_doctype().force_quirks = true;
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::BogusDOCTYPEState);
                    }
                    None => {
                        self.get_current_doctype().force_quirks = true;
                        self.emit_current_token();
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::DOCTYPESystemIdentifierDoublequotedState => {
                match self.read_next() {
                    Some('"') => {
                        self.switch_to(TokenizerState::AfterDOCTYPESystemIdentifierState);
                    }
                    Some('\0') => {
                        // unexpected null character parse error
                        self.get_current_doctype()
                            .system_ident
                            .as_mut()
                            .map(|ident| ident.push(UNICODE_REPLACEMENT));
                    }
                    Some('>') => {
                        // abrupt doctype system identifier parse error
                        self.get_current_doctype().force_quirks = true;
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    Some(c) => {
                        self.get_current_doctype()
                            .system_ident
                            .as_mut()
                            .map(|ident| ident.push(c));
                    }
                    None => {
                        self.get_current_doctype().force_quirks = true;
                        self.emit_current_token();
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::DOCTYPESystemIdentifierSinglequotedState => {
                match self.read_next() {
                    Some('\'') => {
                        self.switch_to(TokenizerState::AfterDOCTYPESystemIdentifierState);
                    }
                    Some('\0') => {
                        // unexpected null character parse error
                        self.get_current_doctype()
                            .system_ident
                            .as_mut()
                            .map(|ident| ident.push(UNICODE_REPLACEMENT));
                    }
                    Some('>') => {
                        // abrupt doctype system identifier parse error
                        self.get_current_doctype().force_quirks = true;
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    Some(c) => {
                        self.get_current_doctype()
                            .system_ident
                            .as_mut()
                            .map(|ident| ident.push(c));
                    }
                    None => {
                        self.get_current_doctype().force_quirks = true;
                        self.emit_current_token();
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::AfterDOCTYPESystemIdentifierState => {
                match self.read_next() {
                    //       tab       line feed    form feed     space
                    Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}') => {}
                    Some('>') => {
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    Some(_) => {
                        // unexpected character after doctype system identifier parse error
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::BogusDOCTYPEState);
                    }
                    None => {
                        // eof in doctype parse error
                        self.get_current_doctype().force_quirks = true;
                        self.emit_current_token();
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::BogusDOCTYPEState => {
                match self.read_next() {
                    Some('>') => {
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    Some('\0') => {
                        // unexpected null character parse error
                    }
                    Some(_) => {}
                    None => {
                        self.emit_current_token();
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::CDATASectionState => {
                match self.read_next() {
                    Some(']') => {
                        self.switch_to(TokenizerState::CDATASectionBracketState);
                    }
                    Some(c) => {
                        self.emit(Token::Character(c));
                    }
                    None => {
                        // eof in cdata parse error
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::CDATASectionBracketState => match self.read_next() {
                Some(']') => {
                    self.switch_to(TokenizerState::CDATASectionEndState);
                }
                _ => {
                    self.emit(Token::Character(']'));
                    self.ptr -= 1;
                    self.switch_to(TokenizerState::CDATASectionState);
                }
            },
            TokenizerState::CDATASectionEndState => match self.read_next() {
                Some(']') => {
                    self.emit(Token::Character(']'));
                }
                Some('>') => {
                    self.switch_to(TokenizerState::DataState);
                }
                _ => {
                    self.emit(Token::Character(']'));
                    self.emit(Token::Character(']'));
                    self.ptr -= 1;
                    self.switch_to(TokenizerState::CDATASectionState);
                }
            },
            TokenizerState::CharacterReferenceState => {
                self.buffer = Some("&".to_string());
                match self.read_next() {
                    Some('a'..='z' | 'A'..='Z' | '0'..='9') => {
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::NamedCharacterReferenceState);
                    }
                    Some('#') => {
                        self.add_to_buffer('#');
                        self.switch_to(TokenizerState::NumericCharacterReferenceState);
                    }
                    _ => {
                        // we are supposed to flush the buffer as tokens - but we just set it to "&".
                        // Let's just emit a single '&' token i guess?
                        // Sorry to future me if this causes any bugs :^)
                        self.emit(Token::Character('&'));
                        self.ptr -= 1;
                        self.switch_to(self.return_state.unwrap());
                    }
                }
            }
            TokenizerState::NamedCharacterReferenceState => {
                match match_reference(&self.source[self.ptr..]) {
                    Some(unicode_val) => {
                        _ = unicode_val;
                        todo!();
                    }
                    None => {
                        self.switch_to(TokenizerState::AmbiguousAmpersandState);
                    }
                }
            }
            TokenizerState::AmbiguousAmpersandState => {
                match self.read_next() {
                    Some(c @ ('a'..='z' | 'A'..='Z' | '0'..='9')) => {
                        let was_consumed_as_part_of_attr = false;
                        if was_consumed_as_part_of_attr {
                            self.get_current_tag().add_to_attr_value(c);
                        } else {
                            self.emit(Token::Character(c));
                        }
                        todo!();
                    }
                    Some(';') => {
                        // unknown named character reference parse error
                        self.ptr -= 1;
                        self.switch_to(self.return_state.unwrap());
                    }
                    _ => {
                        self.ptr -= 1;
                        self.switch_to(self.return_state.unwrap());
                    }
                }
            }
            TokenizerState::NumericCharacterReferenceState => {
                self.character_reference_code = 0;
                match self.read_next() {
                    Some(c @ ('X' | 'x')) => {
                        self.add_to_buffer(c);
                        self.switch_to(TokenizerState::HexadecimalCharacterReferenceStartState);
                    }
                    _ => {
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::DecimalCharacterReferenceStartState);
                    }
                }
            }
            TokenizerState::HexadecimalCharacterReferenceStartState => {
                match self.read_next() {
                    Some('0'..='9' | 'a'..='f' | 'A'..='F') => {
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::HexadecimalCharacterReferenceState);
                    }
                    _ => {
                        // absence of digits in numeric character reference parse error
                        // flush code points consumed as a character reference
                        self.ptr -= 1;
                        self.switch_to(self.return_state.unwrap());
                        todo!();
                    }
                }
            }
            TokenizerState::DecimalCharacterReferenceStartState => {
                match self.read_next() {
                    Some('0'..='9') => {
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::DecimalCharacterReferenceState);
                    }
                    _ => {
                        // absence of digits in numeric character reference parse error
                        // flush code points consumed as a character reference
                        self.ptr -= 1;
                        self.switch_to(self.return_state.unwrap());
                        todo!();
                    }
                }
            }
            TokenizerState::HexadecimalCharacterReferenceState => {
                match self.read_next() {
                    Some(c @ ('0'..='9' | 'a'..='f' | 'A'..='F')) => {
                        self.character_reference_code *= 16;
                        self.character_reference_code += c.to_digit(16).unwrap();
                    }
                    Some(';') => {
                        self.switch_to(TokenizerState::NumericCharacterReferenceEndState);
                    }
                    _ => {
                        // missing semicolon after character reference parse error
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::NumericCharacterReferenceEndState);
                    }
                }
            }
            TokenizerState::DecimalCharacterReferenceState => {
                match self.read_next() {
                    Some(c @ '0'..='9') => {
                        self.character_reference_code *= 10;
                        self.character_reference_code += c.to_digit(10).unwrap();
                    }
                    Some(';') => {
                        self.switch_to(TokenizerState::NumericCharacterReferenceEndState);
                    }
                    _ => {
                        // missing semicolon after character reference parse error
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::NumericCharacterReferenceEndState);
                    }
                }
            }
            TokenizerState::NumericCharacterReferenceEndState => {
                match self.character_reference_code {
                    0x00 => {
                        // null character reference parse error
                        self.character_reference_code = 0xFFFD;
                    }
                    0x110000.. => {
                        // character reference outside unicode range parse error
                        self.character_reference_code = 0xFFFD;
                    }
                    // check for surrogates
                    0xD800..=0xDFFF => {
                        // surrogate character reference parse error
                        self.character_reference_code = 0xFFFD;
                    }
                    // check for noncharacters
                    0xFDD0..=0xFDEF
                    | 0x0FFFE
                    | 0x0FFFF
                    | 0x1FFFE
                    | 0x1FFFF
                    | 0x2FFFE
                    | 0x2FFFF
                    | 0x3FFFE
                    | 0x3FFFF
                    | 0x4FFFE
                    | 0x4FFFF
                    | 0x5FFFE
                    | 0x5FFFF
                    | 0x6FFFE
                    | 0x6FFFF
                    | 0x7FFFE
                    | 0x7FFFF
                    | 0x8FFFE
                    | 0x8FFFF
                    | 0x9FFFE
                    | 0x9FFFF
                    | 0xAFFFE
                    | 0xAFFFF
                    | 0xBFFFE
                    | 0xBFFFF
                    | 0xCFFFE
                    | 0xCFFFF
                    | 0xDFFFE
                    | 0xDFFFF
                    | 0xEFFFE
                    | 0xEFFFF
                    | 0xFFFFE
                    | 0xFFFFF
                    | 0x10FFFE
                    | 0x10FFFF => {
                        // noncharacter character reference parse error
                        self.character_reference_code = 0xFFFD;
                    }
                    c @ (0x0D | 0xC0 | 0x007F..=0x009F) => {
                        //       tab       line feed    form feed     space
                        if c != 0x009 || c != 0x000A || c != 0x000C || c != 0x0020 {
                            // control character reference parse error
                            match c {
                                0x80 => {
                                    self.character_reference_code = 0x20AC;
                                }
                                0x82 => {
                                    self.character_reference_code = 0x201A;
                                }
                                0x83 => {
                                    self.character_reference_code = 0x0192;
                                }
                                0x84 => {
                                    self.character_reference_code = 0x201E;
                                }
                                0x85 => {
                                    self.character_reference_code = 0x2026;
                                }
                                0x86 => {
                                    self.character_reference_code = 0x2020;
                                }
                                0x87 => {
                                    self.character_reference_code = 0x2021;
                                }
                                0x88 => {
                                    self.character_reference_code = 0x02C6;
                                }
                                0x89 => {
                                    self.character_reference_code = 0x2030;
                                }
                                0x8A => {
                                    self.character_reference_code = 0x0160;
                                }
                                0x8B => {
                                    self.character_reference_code = 0x2039;
                                }
                                0x8C => {
                                    self.character_reference_code = 0x0152;
                                }
                                0x8E => {
                                    self.character_reference_code = 0x017D;
                                }
                                0x91 => {
                                    self.character_reference_code = 0x2018;
                                }
                                0x92 => {
                                    self.character_reference_code = 0x2019;
                                }
                                0x93 => {
                                    self.character_reference_code = 0x201C;
                                }
                                0x94 => {
                                    self.character_reference_code = 0x201D;
                                }
                                0x95 => {
                                    self.character_reference_code = 0x2022;
                                }
                                0x96 => {
                                    self.character_reference_code = 0x2013;
                                }
                                0x97 => {
                                    self.character_reference_code = 0x2014;
                                }
                                0x98 => {
                                    self.character_reference_code = 0x02DC;
                                }
                                0x99 => {
                                    self.character_reference_code = 0x2122;
                                }
                                0x9A => {
                                    self.character_reference_code = 0x0161;
                                }
                                0x9B => {
                                    self.character_reference_code = 0x203A;
                                }
                                0x9C => {
                                    self.character_reference_code = 0x0153;
                                }
                                0x9E => {
                                    self.character_reference_code = 0x017E;
                                }
                                0x9F => {
                                    self.character_reference_code = 0x0178;
                                }
                                _ => {} // no mapping
                            }
                        }
                    }
                    _ => {}
                }
                self.buffer = Some(
                    char::from_u32(self.character_reference_code)
                        .unwrap()
                        .to_string(),
                );
                self.switch_to(self.return_state.unwrap());
                todo!(); // flush, again
            }
        }
    }
}

impl Iterator for Tokenizer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            None
        } else {
            while self.token_buffer.is_empty() {
                self.step();
            }
            let first_token = self.token_buffer.pop_front();
            if let Some(Token::EOF) = first_token {
                self.done = true;
            }
            first_token
        }
    }
}
