use super::character_reference::match_reference;
use std::collections::VecDeque;

const UNICODE_REPLACEMENT: char = '\u{FFFD}';

#[derive(Debug, Clone, Copy)]
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
                // Consume the next input character:
                match self.read_next() {
                    Some('&') => {
                        // Set the return state to the data state. Switch to the character
                        // reference state.
                        self.return_state = Some(TokenizerState::DataState);
                        self.switch_to(TokenizerState::CharacterReferenceState);
                    }
                    Some('<') => {
                        // Switch to the tag open state.
                        self.switch_to(TokenizerState::TagOpenState);
                    }
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        // Emit the current input character as a character token.
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    }
                    Some(c) => {
                        // Emit the current input character as a character token.
                        self.emit(Token::Character(c));
                    }
                    None => {
                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::RCDATAState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('&') => {
                        // Set the return state to the RCDATA state. Switch to the character
                        // reference state.
                        self.return_state = Some(TokenizerState::RCDATAState);
                        self.switch_to(TokenizerState::CharacterReferenceState);
                    }
                    Some('<') => {
                        // Switch to the RCDATA less-than sign state.
                        self.switch_to(TokenizerState::RCDATALessThanSignState);
                    }
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        // Emit the current input character as a character token.
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    }
                    Some(c) => {
                        // Emit the current input character as a character token.
                        self.emit(Token::Character(c));
                    }
                    None => {
                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::RAWTEXTState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('<') => {
                        // Switch to the RAWTEXT less-than sign state.
                        self.switch_to(TokenizerState::RAWTEXTLessThanSignState);
                    }
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        // Emit the current input character as a character token.
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    }
                    Some(c) => {
                        // Emit the current input character as a character token.
                        self.emit(Token::Character(c));
                    }
                    None => {
                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::ScriptDataState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('<') => {
                        // Switch to the script data less-than sign state.
                        self.switch_to(TokenizerState::ScriptDataLessThanSignState);
                    }
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        // Emit the current input character as a character token.
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    }
                    Some(c) => {
                        // Emit the current input character as a character token.
                        self.emit(Token::Character(c));
                    }
                    None => {
                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::PLAINTEXTState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        // Emit the current input character as a character token.
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    }
                    Some(c) => {
                        // Emit the current input character as a character token.
                        self.emit(Token::Character(c));
                    }
                    None => {
                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::TagOpenState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('!') => {
                        // Switch to the markup declaration open state.
                        self.switch_to(TokenizerState::MarkupDeclarationOpenState);
                    }
                    Some('/') => {
                        //     Switch to the end tag open state.
                        self.switch_to(TokenizerState::EndTagOpenState);
                    }
                    Some('a'..='z' | 'A'..='Z') => {
                        // Create a new start tag token, set its tag name to the empty string.
                        // Reconsume in the tag name state.
                        self.current_token = Some(Token::Tag(TagData::default_open()));
                        self.ptr -= 1; // reconsume the current character
                        self.switch_to(TokenizerState::TagNameState);
                    }
                    Some('?') => {
                        // This is an unexpected-question-mark-instead-of-tag-name parse error.
                        // Create a comment token whose data is the empty string. Reconsume in the
                        // bogus comment state.
                        self.current_token = Some(Token::Comment(String::default()));
                        self.ptr -= 1; // reconsume current character
                        self.switch_to(TokenizerState::BogusCommentState);
                    }
                    Some(_) => {
                        // This is an invalid-first-character-of-tag-name parse error. Emit a
                        // U+003C LESS-THAN SIGN character token. Reconsume in the data state.
                        self.emit(Token::Character('<'));
                        self.ptr -= 1; // reconsume current token
                        self.switch_to(TokenizerState::DataState);
                    }
                    None => {
                        // This is an eof-before-tag-name parse error. Emit a U+003C LESS-THAN SIGN
                        // character token and an end-of-file token.
                        self.emit(Token::Character('<'));
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::EndTagOpenState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('a'..='z' | 'A'..='Z') => {
                        // Create a new end tag token, set its tag name to the empty string.
                        // Reconsume in the tag name state.
                        self.current_token = Some(Token::Tag(TagData::default_close()));
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::TagNameState);
                    }
                    Some('>') => {
                        // This is a missing-end-tag-name parse error. Switch to the data state.
                        self.switch_to(TokenizerState::DataState);
                    }
                    Some(_) => {
                        // This is an invalid-first-character-of-tag-name parse error. Create a
                        // comment token whose data is the empty string. Reconsume in the bogus
                        // comment state.
                        self.current_token = Some(Token::Comment(String::default()));
                        self.ptr -= 1; // reconsume current character
                        self.switch_to(TokenizerState::BogusCommentState);
                    }
                    None => {
                        // This is an eof-before-tag-name parse error. Emit a U+003C LESS-THAN SIGN
                        // character token, a U+002F SOLIDUS character token and an end-of-file
                        // token.
                        self.emit(Token::Character('<'));
                        self.emit(Token::Character('/'));
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::TagNameState => {
                // Consume the next input character:
                match self.read_next() {
                    //       tab       line feed    form feed     space
                    Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}') => {
                        // Switch to the before attribute name state.
                        self.switch_to(TokenizerState::BeforeAttributeNameState);
                    }
                    Some('/') => {
                        // Switch to the self-closing start tag state.
                        self.switch_to(TokenizerState::SelfClosingStartTagState);
                    }
                    Some('>') => {
                        // Switch to the data state. Emit the current tag token.
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    Some(mut c @ 'A'..='Z') => {
                        // Append the lowercase version of the current input character (add
                        // 0x0020 to the character's code point) to the current tag token's tag
                        // name.
                        c.make_ascii_lowercase();
                        self.get_current_tag().name.push(c);
                    }
                    Some('\0') => {
                        // This is an unexpected-null-character parse error. Append a U+FFFD
                        // REPLACEMENT CHARACTER character to the current tag token's tag name.
                        self.get_current_tag().name.push(UNICODE_REPLACEMENT);
                    }
                    Some(c) => {
                        // Append the current input character to the current tag token's tag name.
                        self.get_current_tag().name.push(c);
                    }
                    None => {
                        // This is an eof-in-tag parse error. Emit an end-of-file token.
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::RCDATALessThanSignState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('/') => {
                        // Set the temporary buffer to the empty string. Switch to the RCDATA end
                        // tag open state.
                        self.buffer = Some(String::new());
                        self.switch_to(TokenizerState::RCDATAEndTagOpenState);
                    }
                    _ => {
                        // Emit a U+003C LESS-THAN SIGN character token. Reconsume in the RCDATA
                        // state.
                        self.emit(Token::Character('<'));
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::RCDATAState);
                    }
                }
            }
            TokenizerState::RCDATAEndTagOpenState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('a'..='z' | 'A'..='Z') => {
                        // Create a new end tag token, set its tag name to the empty string.
                        // Reconsume in the RCDATA end tag name state.
                        self.current_token = Some(Token::Tag(TagData::default_close()));
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::RCDATAEndTagNameState);
                    }
                    _ => {
                        // Emit a U+003C LESS-THAN SIGN character token and a U+002F SOLIDUS
                        // character token. Reconsume in the RCDATA state.
                        self.emit(Token::Character('<'));
                        self.emit(Token::Character('/'));
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::RCDATAState);
                    }
                }
            }
            TokenizerState::RCDATAEndTagNameState => {
                // Consume the next input character:
                match (self.read_next(), self.is_appropriate_end_token()) {
                    //       tab       line feed    form feed     space
                    (Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}'), true) => {
                        // Switch to the before attribute name state.
                        self.switch_to(TokenizerState::BeforeAttributeNameState);
                    }
                    (Some('/'), true) => {
                        // Switch to the self-closing start tag state.
                        self.switch_to(TokenizerState::SelfClosingStartTagState);
                    }
                    (Some('>'), true) => {
                        // Switch to the data state and emit the current tag token
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    (Some(mut c @ 'A'..='Z'), _) => {
                        // Append the lowercase version of the current input character (add 0x0020
                        // to the character's code point) to the current tag token's tag name.
                        // Append the current input character to the temporary buffer.
                        self.add_to_buffer(c);
                        c.make_ascii_lowercase();
                        self.get_current_tag().name.push(c);
                    }
                    (Some(c @ 'a'..='z'), _) => {
                        // Append the current input character to the current tag token's tag name.
                        // Append the current input character to the temporary buffer.
                        self.add_to_buffer(c);
                        self.get_current_tag().name.push(c);
                    }
                    _ => {
                        // Emit a U+003C LESS-THAN SIGN character token, a U+002F SOLIDUS character
                        // token, and a character token for each of the characters in the temporary
                        // buffer (in the order they were added to the buffer). Reconsume in the
                        // RCDATA state.
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
            TokenizerState::RAWTEXTLessThanSignState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('/') => {
                        // Set the temporary buffer to the empty string. Switch to the RAWTEXT end
                        // tag open state.
                        self.buffer = Some(String::new());
                        self.switch_to(TokenizerState::RAWTEXTEndTagOpenState);
                    }
                    _ => {
                        // Emit a U+003C LESS-THAN SIGN character token. Reconsume in the RAWTEXT
                        // state.
                        self.emit(Token::Character('<'));
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::RAWTEXTState);
                    }
                }
            }
            TokenizerState::RAWTEXTEndTagOpenState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('a'..='z' | 'A'..='Z') => {
                        // Create a new end tag token, set its tag name to the empty string.
                        // Reconsume in the RAWTEXT end tag name state.
                        self.current_token = Some(Token::Tag(TagData::default_close()));
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::RAWTEXTEndTagNameState);
                    }
                    _ => {
                        // Emit a U+003C LESS-THAN SIGN character token and a U+002F SOLIDUS
                        // character token. Reconsume in the RAWTEXT state.
                        self.emit(Token::Character('<'));
                        self.emit(Token::Character('/'));
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::RAWTEXTState);
                    }
                }
            }
            TokenizerState::RAWTEXTEndTagNameState => {
                // Consume the next input character:
                match (self.read_next(), self.is_appropriate_end_token()) {
                    //       tab       line feed    form feed     space
                    (Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}'), true) => {
                        // Switch to the before attribute name state
                        self.switch_to(TokenizerState::BeforeAttributeNameState);
                    }
                    (Some('/'), true) => {
                        // Switch to the self-closing start tag state
                        self.switch_to(TokenizerState::SelfClosingStartTagState);
                    }
                    (Some('>'), true) => {
                        // Switch to the data state and emit the current tag token
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    (Some(mut c @ 'A'..='Z'), _) => {
                        // Append the lowercase version of the current input character (add 0x0020
                        // to the character's code point) to the current tag token's tag name.
                        // Append the current input character to the temporary buffer.
                        self.add_to_buffer(c);
                        c.make_ascii_lowercase();
                        self.get_current_tag().name.push(c);
                    }
                    (Some(c @ 'a'..='z'), _) => {
                        // Append the current input character to the current tag token's tag name.
                        // Append the current input character to the temporary buffer.
                        self.add_to_buffer(c);
                        self.get_current_tag().name.push(c);
                    }
                    _ => {
                        // Emit a U+003C LESS-THAN SIGN character token, a U+002F SOLIDUS character
                        // token, and a character token for each of the characters in the temporary
                        // buffer (in the order they were added to the buffer). Reconsume in the
                        // RAWTEXT state.
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
            TokenizerState::ScriptDataLessThanSignState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('/') => {
                        // Set the temporary buffer to the empty string. Switch to the script data
                        // end tag open state.
                        self.buffer = Some(String::new());
                        self.switch_to(TokenizerState::ScriptDataEndTagOpenState);
                    }
                    Some('!') => {
                        // Switch to the script data escape start state. Emit a U+003C LESS-THAN
                        // SIGN character token and a U+0021 EXCLAMATION MARK character token.
                        self.switch_to(TokenizerState::ScriptDataEscapeStartState);
                        self.emit(Token::Character('<'));
                        self.emit(Token::Character('!'));
                    }
                    _ => {
                        // Emit a U+003C LESS-THAN SIGN character token. Reconsume in the script
                        // data state.
                        self.emit(Token::Character('<'));
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::ScriptDataState);
                    }
                }
            }
            TokenizerState::ScriptDataEndTagOpenState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('a'..='z' | 'A'..='Z') => {
                        // Create a new end tag token, set its tag name to the empty string.
                        // Reconsume in the script data end tag name state.
                        self.current_token = Some(Token::Tag(TagData::default_close()));
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::ScriptDataEndTagNameState);
                    }
                    _ => {
                        // Emit a U+003C LESS-THAN SIGN character token and a U+002F SOLIDUS
                        // character token. Reconsume in the script data state.
                        self.emit(Token::Character('<'));
                        self.emit(Token::Character('/'));
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::ScriptDataState);
                    }
                }
            }
            TokenizerState::ScriptDataEndTagNameState => {
                // Consume the next input character:
                match (self.read_next(), self.is_appropriate_end_token()) {
                    //       tab       line feed    form feed     space
                    (Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}'), true) => {
                        // Switch to the before attribute name state
                        self.switch_to(TokenizerState::BeforeAttributeNameState);
                    }
                    (Some('/'), true) => {
                        // Switch to the self-closing start tag state
                        self.switch_to(TokenizerState::SelfClosingStartTagState);
                    }
                    (Some('>'), true) => {
                        // Switch to the data state and emit the current tag token
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    (Some(mut c @ 'A'..='Z'), _) => {
                        // Append the lowercase version of the current input character (add
                        // 0x0020 to the character's code point) to the current tag token's tag
                        // name. Append the current input character to the temporary buffer.
                        self.add_to_buffer(c);
                        c.make_ascii_lowercase();
                        self.get_current_tag().name.push(c);
                    }
                    (Some(c @ 'a'..='z'), _) => {
                        // Append the current input character to the current tag token's tag name.
                        // Append the current input character to the temporary buffer.
                        self.add_to_buffer(c);
                        self.get_current_tag().name.push(c);
                    }
                    _ => {
                        // Emit a U+003C LESS-THAN SIGN character token, a U+002F SOLIDUS character
                        // token, and a character token for each of the characters in the temporary
                        // buffer (in the order they were added to the buffer). Reconsume in the
                        // script data state.
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
            TokenizerState::ScriptDataEscapeStartState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('-') => {
                        // Switch to the script data escape start dash state. Emit a U+002D
                        // HYPHEN-MINUS character token.
                        self.switch_to(TokenizerState::ScriptDataEscapeStartDashState);
                        self.emit(Token::Character('-'));
                    }
                    _ => {
                        // Reconsume in the script data state.
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::ScriptDataState);
                    }
                }
            }
            TokenizerState::ScriptDataEscapeStartDashState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('-') => {
                        // Switch to the script data escaped dash dash state. Emit a U+002D
                        // HYPHEN-MINUS character token.
                        self.switch_to(TokenizerState::ScriptDataEscapedDashDashState);
                        self.emit(Token::Character('-'));
                    }
                    _ => {
                        // Reconsume in the script data state.
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::ScriptDataState);
                    }
                }
            }
            TokenizerState::ScriptDataEscapedState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('-') => {
                        // Switch to the script data escaped dash state. Emit a U+002D HYPHEN-MINUS
                        // character token.
                        self.switch_to(TokenizerState::ScriptDataEscapedDashState);
                        self.emit(Token::Character('-'));
                    }
                    Some('<') => {
                        // Switch to the script data escaped less-than sign state.
                        self.switch_to(TokenizerState::ScriptDataEscapedLessThanSignState);
                    }
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        // Emit a U+FFFD REPLACEMENT CHARACTER character token.
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    }
                    Some(c) => {
                        // Emit the current input character as a character token.
                        self.emit(Token::Character(c));
                    }
                    None => {
                        // This is an eof-in-script-html-comment-like-text parse error. Emit an
                        // end-of-file token.
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::ScriptDataEscapedDashState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('-') => {
                        // Switch to the script data escaped dash dash state. Emit a U+002D
                        // HYPHEN-MINUS character token.
                        self.switch_to(TokenizerState::ScriptDataEscapedDashDashState);
                        self.emit(Token::Character('-'));
                    }
                    Some('<') => {
                        // Switch to the script data escaped less-than sign state.
                        self.switch_to(TokenizerState::ScriptDataEscapedLessThanSignState);
                    }
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        // Emit a U+FFFD REPLACEMENT CHARACTER character token.
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    }
                    Some(c) => {
                        // Switch to the script data escaped state. Emit the current input
                        // character as a character token.
                        self.switch_to(TokenizerState::ScriptDataEscapedState);
                        self.emit(Token::Character(c));
                    }
                    None => {
                        // This is an eof-in-script-html-comment-like-text parse error. Emit an
                        // end-of-file token.
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::ScriptDataEscapedDashDashState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('-') => {
                        // Emit a U+002D HYPHEN-MINUS character token.
                        self.emit(Token::Character('-'));
                    }
                    Some('<') => {
                        // Switch to the script data escaped less-than sign state.
                        self.switch_to(TokenizerState::ScriptDataEscapedLessThanSignState);
                    }
                    Some('>') => {
                        // Switch to the script data state. Emit a U+003E GREATER-THAN SIGN
                        // character token.
                        self.switch_to(TokenizerState::ScriptDataState);
                        self.emit(Token::Character('>'));
                    }
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        // Emit a U+FFFD REPLACEMENT CHARACTER character token.
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    }
                    Some(c) => {
                        // Switch to the script data escaped state. Emit the current input
                        // character as a character token.
                        self.switch_to(TokenizerState::ScriptDataEscapedState);
                        self.emit(Token::Character(c));
                    }
                    None => {
                        // This is an eof-in-script-html-comment-like-text parse error. Emit an
                        // end-of-file token.
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::ScriptDataEscapedLessThanSignState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('/') => {
                        // Set the temporary buffer to the empty string. Switch to the script data
                        // escaped end tag open state.
                        self.buffer = Some(String::new());
                        self.switch_to(TokenizerState::ScriptDataEscapedEndTagOpenState);
                    }
                    Some('a'..='z' | 'A'..='Z') => {
                        // Set the temporary buffer to the empty string. Emit a U+003C LESS-THAN
                        // SIGN character token. Reconsume in the script data double escape start
                        // state.
                        self.buffer = Some(String::new());
                        self.emit(Token::Character('<'));
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::ScriptDataDoubleEscapeStartState);
                    }
                    _ => {
                        // Emit a U+003C LESS-THAN SIGN character token. Reconsume in the
                        // script data escaped state.
                        self.emit(Token::Character('<'));
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::ScriptDataEscapedState);
                    }
                }
            }
            TokenizerState::ScriptDataEscapedEndTagOpenState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('a'..='z' | 'A'..='Z') => {
                        // Create a new end tag token, set its tag name to the empty string.
                        // Reconsume in the script data escaped end tag name state.
                        self.current_token = Some(Token::Tag(TagData::default_close()));
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::ScriptDataEscapedEndTagNameState);
                    }
                    _ => {
                        // Emit a U+003C LESS-THAN SIGN character token and a U+002F SOLIDUS
                        // character token. Reconsume in the script data escaped state.
                        self.emit(Token::Character('<'));
                        self.emit(Token::Character('/'));
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::ScriptDataEscapedState);
                    }
                }
            }
            TokenizerState::ScriptDataEscapedEndTagNameState => {
                // Consume the next input character:
                match (self.read_next(), self.is_appropriate_end_token()) {
                    //       tab       line feed    form feed     space
                    (Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}'), true) => {
                        // Switch to the before attribute name state.
                        self.switch_to(TokenizerState::BeforeAttributeNameState);
                    }
                    (Some('/'), true) => {
                        // Switch to the self-closing start tag state.
                        self.switch_to(TokenizerState::SelfClosingStartTagState);
                    }
                    (Some('>'), true) => {
                        // Switch to the data state and emit the current tag token.
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    (Some(mut c @ 'A'..='Z'), _) => {
                        // Append the lowercase version of the current input character (add 0x0020
                        // to the character's code point) to the current tag token's tag name.
                        // Append the current input character to the temporary buffer.
                        self.add_to_buffer(c);
                        c.make_ascii_lowercase();
                        self.get_current_tag().name.push(c);
                    }
                    (Some(c @ 'a'..='z'), _) => {
                        // Append the current input character to the current tag token's tag name.
                        // Append the current input character to the temporary buffer.
                        self.add_to_buffer(c);
                        self.get_current_tag().name.push(c);
                    }
                    _ => {
                        // Emit a U+003C LESS-THAN SIGN character token, a U+002F SOLIDUS character
                        // token, and a character token for each of the characters in the temporary
                        // buffer (in the order they were added to the buffer). Reconsume in the
                        // script data escaped state.
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
                // Consume the next input character:
                match self.read_next() {
                    //             tab       line feed    form feed     space
                    Some(c @ ('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' | '/' | '>')) => {
                        // If the temporary buffer is the string "script",
                        if self.buffer.contains(&"script") {
                            // then switch to the script data double escaped state.
                            self.switch_to(TokenizerState::ScriptDataDoubleEscapedState);
                        } else {
                            // Otherwise, switch to the script data escaped state.
                            self.switch_to(TokenizerState::ScriptDataEscapedState);
                        }
                        // Emit the current input character as a character token.
                        self.emit(Token::Character(c));
                    }
                    Some(mut c @ 'A'..='Z') => {
                        // Append the lowercase version of the current input character (add 0x0020
                        // to the character's code point) to the temporary buffer. Emit the current
                        // input character as a character token.
                        self.emit(Token::Character(c));
                        c.make_ascii_lowercase();
                        self.add_to_buffer(c);
                    }
                    Some(c @ 'a'..='z') => {
                        // Append the current input character to the temporary buffer. Emit the
                        // current input character as a character token.
                        self.add_to_buffer(c);
                        self.emit(Token::Character(c));
                    }
                    _ => {
                        // Reconsume in the script data escaped state.
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::ScriptDataEscapedState);
                    }
                }
            }
            TokenizerState::ScriptDataDoubleEscapedState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('-') => {
                        // Switch to the script data double escaped dash state. Emit a U+002D
                        // HYPHEN-MINUS character token.
                        self.switch_to(TokenizerState::ScriptDataDoubleEscapedDashState);
                        self.emit(Token::Character('-'));
                    }
                    Some('<') => {
                        // Switch to the script data double escaped less-than sign state. Emit a
                        // U+003C LESS-THAN SIGN character token.
                        self.switch_to(TokenizerState::ScriptDataDoubleEscapedLessThanSignState);
                        self.emit(Token::Character('<'));
                    }
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        // Emit a U+FFFD REPLACEMENT CHARACTER character token.
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    }
                    Some(c) => {
                        // Emit the current input character as a character token.
                        self.emit(Token::Character(c));
                    }
                    None => {
                        // This is an eof-in-script-html-comment-like-text parse error. Emit an
                        // end-of-file token.
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::ScriptDataDoubleEscapedDashState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('-') => {
                        // Switch to the script data double escaped dash dash state. Emit a U+002D
                        // HYPHEN-MINUS character token.
                        self.switch_to(TokenizerState::ScriptDataDoubleEscapedDashDashState);
                        self.emit(Token::Character('-'));
                    }
                    Some('<') => {
                        // This is an unexpected-null-character parse error. Switch to the script
                        // data double escaped state. Emit a U+FFFD REPLACEMENT CHARACTER character
                        // token.
                        self.switch_to(TokenizerState::ScriptDataDoubleEscapedState);
                        self.emit(Token::Character('<'));
                    }
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        // Switch to the script data double escaped state.
                        // Emit a U+FFFD REPLACEMENT CHARACTER character token.
                        self.switch_to(TokenizerState::ScriptDataDoubleEscapedState);
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    }
                    Some(c) => {
                        // Switch to the script data double escaped state. Emit the current input
                        // character as a character token.
                        self.switch_to(TokenizerState::ScriptDataDoubleEscapedState);
                        self.emit(Token::Character(c));
                    }
                    None => {
                        // This is an eof-in-script-html-comment-like-text parse error. Emit an
                        // end-of-file token.
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::ScriptDataDoubleEscapedDashDashState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('-') => {
                        // Emit a U+002D HYPHEN-MINUS character token.
                        self.emit(Token::Character('-'));
                    }
                    Some('<') => {
                        // Switch to the script data double escaped less-than sign state. Emit a
                        // U+003C LESS-THAN SIGN character token.
                        self.switch_to(TokenizerState::ScriptDataDoubleEscapedLessThanSignState);
                        self.emit(Token::Character('<'));
                    }
                    Some('>') => {
                        // Switch to the script data state. Emit a U+003E GREATER-THAN SIGN
                        // character token.
                        self.switch_to(TokenizerState::ScriptDataState);
                        self.emit(Token::Character('>'));
                    }
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        // Switch to the script data double escaped state.
                        // Emit a U+FFFD REPLACEMENT CHARACTER character token.
                        self.switch_to(TokenizerState::ScriptDataDoubleEscapedState);
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    }
                    Some(c) => {
                        // Switch to the script data double escaped state. Emit the current input
                        // character as a character token.
                        self.switch_to(TokenizerState::ScriptDataDoubleEscapedState);
                        self.emit(Token::Character(c));
                    }
                    None => {
                        // This is an eof-in-script-html-comment-like-text parse error. Emit an
                        // end-of-file token.
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::ScriptDataDoubleEscapedLessThanSignState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('/') => {
                        // Set the temporary buffer to the empty string. Switch to the script data
                        // double escape end state. Emit a U+002F SOLIDUS character token.
                        self.buffer = Some(String::new());
                        self.switch_to(TokenizerState::ScriptDataDoubleEscapeEndState);
                        self.emit(Token::Character('/'));
                    }
                    _ => {
                        // Reconsume in the script data double escaped state.
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::ScriptDataDoubleEscapedState);
                    }
                }
            }
            TokenizerState::ScriptDataDoubleEscapeEndState => {
                // Consume the next input character:
                match self.read_next() {
                    //            tab       line feed    form feed     space
                    Some(c @ ('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' | '/' | '>')) => {
                        // If the temporary buffer is the string "script",
                        if self.buffer.contains(&"script") {
                            // then switch to the script data escaped state.
                            self.switch_to(TokenizerState::ScriptDataEscapedState);
                        } else {
                            // Otherwise, switch to the script data double escaped state.
                            self.switch_to(TokenizerState::ScriptDataDoubleEscapedState);
                        }
                        // Emit the current input character as a character token.
                        self.emit(Token::Character(c));
                    }
                    Some(mut c @ 'A'..='Z') => {
                        // Append the lowercase version of the current input character (add
                        // 0x0020 to the character's code point) to the temporary buffer. Emit
                        // the current input character as a character token.
                        self.emit(Token::Character(c));
                        c.make_ascii_lowercase();
                        self.add_to_buffer(c);
                    }
                    Some(c @ 'a'..='z') => {
                        // Append the current input character to the temporary buffer. Emit the
                        // current input character as a character token.
                        self.add_to_buffer(c);
                        self.emit(Token::Character(c));
                    }
                    _ => {
                        // Reconsume in the script data double escaped state.
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::ScriptDataDoubleEscapedState);
                    }
                }
            }
            TokenizerState::BeforeAttributeNameState => {
                // Consume the next input character:
                match self.read_next() {
                    //      tab       line feed    form feed     space
                    Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}') => {} // Ignore the character.
                    Some('/' | '>') | None => {
                        // Reconsume in the after attribute name state.
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::AfterAttributeNameState);
                    }
                    Some('=') => {
                        // This is an unexpected-equals-sign-before-attribute-name parse error.
                        // Start a new attribute in the current tag token. Set that attribute's
                        // name to the current input character, and its value to the empty string.
                        // Switch to the attribute name state.
                        self.get_current_tag().new_attribute();
                        self.get_current_tag().add_to_attr_name('=');
                        self.switch_to(TokenizerState::AttributeNameState);
                    }
                    _ => {
                        // Start a new attribute in the current tag token. Set that attribute name
                        // and value to the empty string. Reconsume in the attribute name state.
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
                //
                // Consume the next input character:
                match self.read_next() {
                    //      tab       line feed    form feed     space
                    Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' | '/' | '>') | None => {
                        // Reconsume in the after attribute name state.
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::AfterAttributeNameState);
                    }
                    Some('=') => {
                        // Switch to the before attribute value state.
                        self.switch_to(TokenizerState::BeforeAttributeValueState);
                    }
                    Some(mut c @ 'A'..='Z') => {
                        // Append the lowercase version of the current input character (add 0x0020
                        // to the character's code point) to the current attribute's name.
                        c.make_ascii_lowercase();
                        self.get_current_tag().add_to_attr_name(c);
                    }
                    Some('\0') => {
                        // This is an unexpected-null-character parse error. Append a U+FFFD
                        // REPLACEMENT CHARACTER character to the current attribute's name.
                        self.get_current_tag().add_to_attr_name(UNICODE_REPLACEMENT);
                    }
                    Some(c @ ('"' | '\'' | '<')) => {
                        // This is an unexpected-character-in-attribute-name parse error. Treat it
                        // as per the "anything else" entry below.
                        self.get_current_tag().add_to_attr_name(c);
                    }
                    Some(c) => {
                        // Append the current input character to the current attribute's name.
                        self.get_current_tag().add_to_attr_name(c);
                    }
                }
            }
            TokenizerState::AfterAttributeNameState => {
                // Consume the next input character:
                match self.read_next() {
                    //      tab       line feed    form feed     space
                    Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}') => {} // Ignore the character.
                    Some('/') => {
                        // Switch to the self-closing start tag state.
                        self.switch_to(TokenizerState::SelfClosingStartTagState);
                    }
                    Some('=') => {
                        // Switch to the before attribute value state.
                        self.switch_to(TokenizerState::BeforeAttributeValueState);
                    }
                    Some('>') => {
                        // Switch to the data state. Emit the current tag token.
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    Some(_) => {
                        // Start a new attribute in the current tag token. Set that attribute name
                        // and value to the empty string. Reconsume in the attribute name state.
                        self.get_current_tag().new_attribute();
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::AttributeNameState);
                    }
                    None => {
                        // This is an eof-in-tag parse error. Emit an end-of-file token.
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::BeforeAttributeValueState => {
                // Consume the next input character:
                match self.read_next() {
                    //      tab       line feed    form feed     space
                    Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}') => {} // Ignore the character.
                    Some('"') => {
                        // Switch to the attribute value (double-quoted) state.
                        self.switch_to(TokenizerState::AttributeValueDoublequotedState);
                    }
                    Some('\'') => {
                        // Switch to the attribute value (single-quoted) state.
                        self.switch_to(TokenizerState::AttributeValueSinglequotedState);
                    }
                    Some('>') => {
                        // This is a missing-attribute-value parse error. Switch to the data state.
                        // Emit the current tag token.
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    _ => {
                        // Reconsume in the attribute value (unquoted) state.
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::AttributeValueUnquotedState);
                    }
                }
            }
            TokenizerState::AttributeValueDoublequotedState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('"') => {
                        // Switch to the after attribute value (quoted) state.
                        self.switch_to(TokenizerState::AfterAttributeValueQuotedState);
                    }
                    Some('&') => {
                        // Set the return state to the attribute value (double-quoted) state.
                        // Switch to the character reference state.
                        self.return_state = Some(TokenizerState::AttributeValueDoublequotedState);
                        self.switch_to(TokenizerState::CharacterReferenceState);
                    }
                    Some('\0') => {
                        // This is an unexpected-null-character parse error. Append a U+FFFD
                        // REPLACEMENT CHARACTER character to the current attribute's value.
                        self.get_current_tag()
                            .add_to_attr_value(UNICODE_REPLACEMENT);
                    }
                    Some(c) => {
                        // Append the current input character to the current attribute's value.
                        self.get_current_tag().add_to_attr_value(c);
                    }
                    None => {
                        // This is an eof-in-tag parse error. Emit an end-of-file token.
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::AttributeValueSinglequotedState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('\'') => {
                        // Switch to the after attribute value (quoted) state.
                        self.switch_to(TokenizerState::AfterAttributeValueQuotedState);
                    }
                    Some('&') => {
                        // Set the return state to the attribute value (single-quoted) state.
                        // Switch to the character reference state.
                        self.return_state = Some(TokenizerState::AttributeValueSinglequotedState);
                        self.switch_to(TokenizerState::CharacterReferenceState);
                    }
                    Some('\0') => {
                        // This is an unexpected-null-character parse error. Append a U+FFFD
                        // REPLACEMENT CHARACTER character to the current attribute's value.
                        self.get_current_tag()
                            .add_to_attr_value(UNICODE_REPLACEMENT);
                    }
                    Some(c) => {
                        // Append the current input character to the current attribute's value.
                        self.get_current_tag().add_to_attr_value(c);
                    }
                    None => {
                        // This is an eof-in-tag parse error. Emit an end-of-file token.
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::AttributeValueUnquotedState => {
                // Consume the next input character:
                match self.read_next() {
                    //      tab       line feed    form feed     space
                    Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}') => {
                        // Switch to the before attribute name state.
                        self.switch_to(TokenizerState::BeforeAttributeNameState);
                    }
                    Some('&') => {
                        // Set the return state to the attribute value (unquoted) state. Switch to
                        // the character reference state.
                        self.return_state = Some(TokenizerState::AttributeValueUnquotedState);
                        self.switch_to(TokenizerState::CharacterReferenceState);
                    }
                    Some('>') => {
                        // Switch to the data state. Emit the current tag token.
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    Some('\0') => {
                        // This is an unexpected-null-character parse error. Append a U+FFFD
                        // REPLACEMENT CHARACTER character to the current attribute's value.
                        self.get_current_tag()
                            .add_to_attr_value(UNICODE_REPLACEMENT);
                    }
                    Some(c @ ('"' | '\'' | '<' | '=' | '`')) => {
                        // This is an unexpected-character-in-unquoted-attribute-value parse error.
                        // Treat it as per the "anything else" entry below.
                        self.get_current_tag().add_to_attr_value(c);
                    }
                    Some(c) => {
                        // Append the current input character to the current attribute's value.
                        self.get_current_tag().add_to_attr_value(c);
                    }
                    None => {
                        // This is an eof-in-tag parse error. Emit an end-of-file token.
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::AfterAttributeValueQuotedState => {
                // Consume the next input character:
                match self.read_next() {
                    //      tab       line feed    form feed     space
                    Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}') => {
                        // Switch to the before attribute name state.
                        self.switch_to(TokenizerState::BeforeAttributeNameState);
                    }
                    Some('/') => {
                        // Switch to the self-closing start tag state.
                        self.switch_to(TokenizerState::SelfClosingStartTagState);
                    }
                    Some('>') => {
                        // Switch to the data state. Emit the current tag token.
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    Some(_) => {
                        // This is a missing-whitespace-between-attributes parse error. Reconsume
                        // in the before attribute name state.
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::BeforeAttributeNameState);
                    }
                    None => {
                        // This is an eof-in-tag parse error. Emit an end-of-file token.
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::SelfClosingStartTagState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('>') => {
                        // Set the self-closing flag of the current tag token. Switch to the data
                        // state. Emit the current tag token.
                        self.get_current_tag().self_closing = true;
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    Some(_) => {
                        // This is an unexpected-solidus-in-tag parse error. Reconsume in the
                        // before attribute name state.
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::BeforeAttributeNameState);
                    }
                    None => {
                        // This is an eof-in-tag parse error. Emit an end-of-file token.
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::BogusCommentState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('>') => {
                        // Switch to the data state. Emit the current comment token.
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    Some('\0') => {
                        // This is an unexpected-null-character parse error. Append a U+FFFD
                        // REPLACEMENT CHARACTER character to the comment token's data.
                        self.get_current_comment().push(UNICODE_REPLACEMENT);
                    }
                    Some(c) => {
                        // Append the current input character to the comment token's data.
                        self.get_current_comment().push(c);
                    }
                    None => {
                        // Emit the comment. Emit an end-of-file token.
                        self.emit_current_token();
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::MarkupDeclarationOpenState => {
                // If the next few characters are:
                if &self.source[self.ptr..self.ptr + 2] == "--" {
                    // Consume those two characters, create a comment token whose data is the empty
                    // string, and switch to the comment start state.
                    self.ptr += 2;
                    self.current_token = Some(Token::Comment(String::default()));
                    self.switch_to(TokenizerState::CommentStartState);
                } else if self.source[self.ptr..self.ptr + 7].eq_ignore_ascii_case("DOCTYPE") {
                    // Consume those characters and switch to the DOCTYPE state.
                    self.ptr += 7;
                    self.switch_to(TokenizerState::DOCTYPEState);
                } else if &self.source[self.ptr..self.ptr + 7] == "[CDATA[" {
                    // Consume those characters. If there is an adjusted current node and it is not
                    // an element in the HTML namespace, then switch to the CDATA section state.
                    // Otherwise, this is a cdata-in-html-content parse error. Create a comment
                    // token whose data is the "[CDATA[" string. Switch to the bogus comment state.
                    self.ptr += 7;
                    todo!();
                } else {
                    // This is an incorrectly-opened-comment parse error. Create a comment token
                    // whose data is the empty string. Switch to the bogus comment state (don't
                    // consume anything in the current state).
                    self.current_token = Some(Token::Comment(String::default()));
                    self.switch_to(TokenizerState::BogusCommentState);
                }
            }
            TokenizerState::CommentStartState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('-') => {
                        // Switch to the comment start dash state.
                        self.switch_to(TokenizerState::CommentStartDashState);
                    }
                    Some('>') => {
                        // This is an abrupt-closing-of-empty-comment parse error. Switch to the
                        // data state. Emit the current comment token.
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    _ => {
                        // Reconsume in the comment state.
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::CommentState);
                    }
                }
            }
            TokenizerState::CommentStartDashState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('-') => {
                        // Switch to the comment end state.
                        self.switch_to(TokenizerState::CommentEndState);
                    }
                    Some('>') => {
                        // This is an abrupt-closing-of-empty-comment parse error. Switch to the
                        // data state. Emit the current comment token.
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    Some(_) => {
                        // Append a U+002D HYPHEN-MINUS character (-) to the comment token's data.
                        // Reconsume in the comment state.
                        self.get_current_comment().push('-');
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::CommentState);
                    }
                    None => {
                        // This is an eof-in-comment parse error. Emit the current comment token.
                        // Emit an end-of-file token.
                        self.emit_current_token();
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::CommentState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('>') => {
                        // Append the current input character to the comment token's data. Switch
                        // to the comment less-than sign state.
                        self.get_current_comment().push('<');
                        self.switch_to(TokenizerState::CommentLessThanSignState);
                    }
                    Some('-') => {
                        // Switch to the comment end dash state.
                        self.switch_to(TokenizerState::CommentEndDashState);
                    }
                    Some('\0') => {
                        // This is an unexpected-null-character parse error. Append a U+FFFD
                        // REPLACEMENT CHARACTER character to the comment token's data.
                        self.get_current_comment().push(UNICODE_REPLACEMENT);
                    }
                    Some(c) => {
                        // Append the current input character to the comment token's data.
                        self.get_current_comment().push(c);
                    }
                    None => {
                        // This is an eof-in-comment parse error. Emit the current comment token.
                        // Emit an end-of-file token.
                        self.emit_current_token();
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::CommentLessThanSignState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('!') => {
                        // Append the current input character to the comment token's data. Switch
                        // to the comment less-than sign bang state.
                        self.get_current_comment().push('!');
                        self.switch_to(TokenizerState::CommentLessThanSignBangState);
                    }
                    Some('<') => {
                        // Append the current input character to the comment token's data.
                        self.get_current_comment().push('<');
                    }
                    _ => {
                        // Reconsume in the comment state.
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::CommentState);
                    }
                }
            }
            TokenizerState::CommentLessThanSignBangState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('-') => {
                        // Switch to the comment less-than sign bang dash state.
                        self.switch_to(TokenizerState::CommentLessThanSignBangDashState);
                    }
                    _ => {
                        // Reconsume in the comment state.
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::CommentState);
                    }
                }
            }
            TokenizerState::CommentLessThanSignBangDashState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('-') => {
                        // Switch to the comment less-than sign bang dash dash state.
                        self.switch_to(TokenizerState::CommentLessThanSignBangDashDashState);
                    }
                    _ => {
                        // Reconsume in the comment end dash state.
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::CommentEndDashState);
                    }
                }
            }
            TokenizerState::CommentLessThanSignBangDashDashState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('>') | None => {
                        // Reconsume in the comment end state.
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::CommentEndState);
                    }
                    Some(_) => {
                        // This is a nested-comment parse error. Reconsume in the comment end
                        // state.
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::CommentEndState);
                    }
                }
            }
            TokenizerState::CommentEndDashState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('-') => {
                        // Switch to the comment end state.
                        self.switch_to(TokenizerState::CommentEndState);
                    }
                    Some(_) => {
                        // Append a U+002D HYPHEN-MINUS character (-) to the comment token's data.
                        // Reconsume in the comment state.
                        self.get_current_comment().push('-');
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::CommentState);
                    }
                    None => {
                        // This is an eof-in-comment parse error. Emit the current comment token.
                        // Emit an end-of-file token.
                        self.emit_current_token();
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::CommentEndState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('>') => {
                        // Switch to the data state. Emit the current comment token.
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    Some('!') => {
                        // Switch to the comment end bang state.
                        self.switch_to(TokenizerState::CommentEndBangState);
                    }
                    Some('-') => {
                        // Append a U+002D HYPHEN-MINUS character (-) to the comment token's data.
                        self.get_current_comment().push('-');
                    }
                    Some(_) => {
                        // Append two U+002D HYPHEN-MINUS characters (-) to the comment token's
                        // data. Reconsume in the comment state.
                        self.get_current_comment().push_str("--");
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::CommentState);
                    }
                    None => {
                        // This is an eof-in-comment parse error. Emit the current comment token.
                        // Emit an end-of-file token.
                        self.emit_current_token();
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::CommentEndBangState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('-') => {
                        // Append two U+002D HYPHEN-MINUS characters (-) and a U+0021 EXCLAMATION
                        // MARK character (!) to the comment token's data. Switch to the comment
                        // end dash state.
                        self.get_current_comment().push_str("--!");
                        self.switch_to(TokenizerState::CommentEndDashState);
                    }
                    Some('>') => {
                        // This is an incorrectly-closed-comment parse error. Switch to the data
                        // state. Emit the current comment token.
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    Some(_) => {
                        // Append two U+002D HYPHEN-MINUS characters (-) and a U+0021 EXCLAMATION
                        // MARK character (!) to the comment token's data. Reconsume in the comment
                        // state.
                        self.get_current_comment().push_str("--!");
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::CommentState);
                    }
                    None => {
                        // This is an eof-in-comment parse error. Emit the current comment
                        // token. Emit an end-of-file token.
                        self.emit_current_token();
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::DOCTYPEState => {
                // Consume the next input character:
                match self.read_next() {
                    //       tab       line feed    form feed     space
                    Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}') => {
                        // Switch to the before DOCTYPE name state.
                        self.switch_to(TokenizerState::BeforeDOCTYPENameState);
                    }
                    Some('>') => {
                        // Reconsume in the before DOCTYPE name state.
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::BeforeDOCTYPENameState);
                    }
                    Some(_) => {
                        // This is a missing-whitespace-before-doctype-name parse error. Reconsume
                        // in the before DOCTYPE name state.
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::BeforeDOCTYPENameState);
                    }
                    None => {
                        // This is an eof-in-doctype parse error. Create a new DOCTYPE token. Set
                        // its force-quirks flag to on. Emit the current token. Emit an end-of-file
                        // token.
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

                // Consume the next input character:
                match self.read_next() {
                    //       tab       line feed    form feed     space
                    Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}') => {} // Ignore the character.
                    Some(c @ 'A'..='Z') => {
                        // Create a new DOCTYPE token. Set the token's name to the lowercase
                        // version of the current input character (add 0x0020 to the
                        // character's code point). Switch to the DOCTYPE name state.
                        doctype_token.name = Some(c.to_ascii_lowercase().to_string());
                        self.current_token = Some(Token::DOCTYPE(doctype_token));
                        self.switch_to(TokenizerState::DOCTYPENameState);
                    }
                    Some('\0') => {
                        // This is an unexpected-null-character parse error. Create a new
                        // DOCTYPE token. Set the token's name to a U+FFFD REPLACEMENT
                        // CHARACTER character. Switch to the DOCTYPE name state.
                        doctype_token.name = Some(UNICODE_REPLACEMENT.to_string());
                        self.current_token = Some(Token::DOCTYPE(doctype_token));
                        self.switch_to(TokenizerState::DOCTYPENameState);
                    }
                    Some('>') => {
                        // This is a missing-doctype-name parse error. Create a new DOCTYPE
                        // token. Set its force-quirks flag to on. Switch to the data state.
                        // Emit the current token.
                        doctype_token.force_quirks = true;
                        self.emit(Token::DOCTYPE(doctype_token));
                        self.switch_to(TokenizerState::DataState);
                    }
                    Some(c) => {
                        // Create a new DOCTYPE token. Set the token's name to the current input
                        // character. Switch to the DOCTYPE name state.
                        doctype_token.name = Some(c.to_string());
                        self.current_token = Some(Token::DOCTYPE(doctype_token));
                        self.switch_to(TokenizerState::DOCTYPENameState);
                    }
                    None => {
                        // This is an eof-in-doctype parse error. Create a new DOCTYPE token. Set
                        // its force-quirks flag to on. Emit the current token. Emit an end-of-file
                        // token.
                        doctype_token.force_quirks = true;
                        self.emit(Token::DOCTYPE(doctype_token));
                        self.emit(Token::EOF)
                    }
                }
            }
            TokenizerState::DOCTYPENameState => {
                // Consume the next input character:
                match self.read_next() {
                    //       tab       line feed    form feed     space
                    Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}') => {
                        // Switch to the after DOCTYPE name state.
                        self.switch_to(TokenizerState::AfterDOCTYPENameState);
                    }
                    Some('>') => {
                        // Switch to the data state. Emit the current DOCTYPE token.
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    Some(c @ 'A'..='Z') => {
                        // Append the lowercase version of the current input character (add 0x0020
                        // to the character's code point) to the current DOCTYPE token's name.
                        self.get_current_doctype()
                            .name
                            .as_mut()
                            .map(|name| name.push(c.to_ascii_lowercase()));
                    }
                    Some('\0') => {
                        // This is an unexpected-null-character parse error. Append a U+FFFD
                        // REPLACEMENT CHARACTER character to the current DOCTYPE token's name.
                        self.get_current_doctype()
                            .name
                            .as_mut()
                            .map(|name| name.push(UNICODE_REPLACEMENT));
                    }
                    Some(c) => {
                        // Append the current input character to the current DOCTYPE token's name.
                        self.get_current_doctype()
                            .name
                            .as_mut()
                            .map(|name| name.push(c));
                    }
                    None => {
                        // This is an eof-in-doctype parse error. Set the current DOCTYPE token's
                        // force-quirks flag to on. Emit the current DOCTYPE token. Emit an
                        // end-of-file token.
                        self.get_current_doctype().force_quirks = true;
                        self.emit_current_token();
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::AfterDOCTYPENameState => {
                // Consume the next input character:
                match self.read_next() {
                    //       tab       line feed    form feed     space
                    Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}') => {} // Ignore the character.
                    Some('>') => {
                        // Switch to the data state. Emit the current DOCTYPE token.
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    None => {
                        // This is an eof-in-doctype parse error. Set the current DOCTYPE token's
                        // force-quirks flag to on. Emit the current DOCTYPE token. Emit an
                        // end-of-file token.
                        self.get_current_doctype().force_quirks = true;
                        self.emit_current_token();
                        self.emit(Token::EOF);
                    }
                    Some(_) => {
                        self.ptr -= 1;
                        // If the six characters starting from the current input character are
                        // an ASCII case-insensitive match for the word "PUBLIC",
                        let next_six_chars = &self.source[self.ptr..self.ptr + 6];
                        if next_six_chars.eq_ignore_ascii_case("PUBLIC") {
                            // then consume those characters and switch
                            // to the after DOCTYPE public keyword state.
                            self.ptr += 6;
                            self.switch_to(TokenizerState::AfterDOCTYPEPublicKeywordState);
                        }
                        // Otherwise, if the six characters starting from the current input
                        // character are an ASCII case-insensitive match for the word
                        // "SYSTEM",
                        else if next_six_chars.eq_ignore_ascii_case("SYSTEM") {
                            // then consume those characters and switch to the after
                            // DOCTYPE system keyword state.
                            self.ptr += 6;
                            self.switch_to(TokenizerState::AfterDOCTYPESystemKeywordState);
                        }
                        // Otherwise, this is an
                        // invalid-character-sequence-after-doctype-name parse error.
                        else {
                            // Set the current DOCTYPE token's force-quirks flag to on.
                            // Reconsume in the bogus DOCTYPE state.
                            self.get_current_doctype().force_quirks = true;
                            // Note: we reconsume, but because we already decremented
                            // self.ptr (above) we don't need to do it again
                            self.switch_to(TokenizerState::BogusDOCTYPEState);
                        }
                    }
                }
            }
            TokenizerState::AfterDOCTYPEPublicKeywordState => {
                // Consume the next input character:
                match self.read_next() {
                    //       tab       line feed    form feed     space
                    Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}') => {
                        // Switch to the before DOCTYPE public identifier state.
                        self.switch_to(TokenizerState::BeforeDOCTYPEPublicIdentifierState);
                    }
                    Some('"') => {
                        // This is a missing-whitespace-after-doctype-public-keyword parse error.
                        // Set the current DOCTYPE token's public identifier to the empty string
                        // (not missing), then switch to the DOCTYPE public identifier
                        // (double-quoted) state.
                        self.get_current_doctype().public_ident = Some(String::new());
                        self.switch_to(TokenizerState::DOCTYPEPublicIdentifierDoublequotedState);
                    }
                    Some('\'') => {
                        // This is a missing-whitespace-after-doctype-public-keyword parse error.
                        // Set the current DOCTYPE token's public identifier to the empty string
                        // (not missing), then switch to the DOCTYPE public identifier
                        // (single-quoted) state.
                        self.get_current_doctype().public_ident = Some(String::new());
                        self.switch_to(TokenizerState::DOCTYPEPublicIdentifierSinglequotedState);
                    }
                    Some('>') => {
                        // This is a missing-doctype-public-identifier parse error. Set the current
                        // DOCTYPE token's force-quirks flag to on. Switch to the data state. Emit
                        // the current DOCTYPE token.
                        self.get_current_doctype().force_quirks = true;
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    Some(_) => {
                        // This is a missing-quote-before-doctype-public-identifier parse error.
                        // Set the current DOCTYPE token's force-quirks flag to on. Reconsume in
                        // the bogus DOCTYPE state.
                        self.get_current_doctype().force_quirks = true;
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::BogusDOCTYPEState);
                    }
                    None => {
                        // This is an eof-in-doctype parse error. Set the current DOCTYPE token's
                        // force-quirks flag to on. Emit the current DOCTYPE token. Emit an
                        // end-of-file token.
                        self.get_current_doctype().force_quirks = true;
                        self.emit_current_token();
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::BeforeDOCTYPEPublicIdentifierState => {
                // Consume the next input character:
                match self.read_next() {
                    //       tab       line feed    form feed     space
                    Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}') => {} // Ignore the character.
                    Some('"') => {
                        // Set the current DOCTYPE token's public identifier to the empty string
                        // (not missing), then switch to the DOCTYPE public identifier
                        // (double-quoted) state.
                        self.get_current_doctype().public_ident = Some(String::new());
                        self.switch_to(TokenizerState::DOCTYPEPublicIdentifierDoublequotedState);
                    }
                    Some('\'') => {
                        // Set the current DOCTYPE token's public identifier to the empty string
                        // (not missing), then switch to the DOCTYPE public identifier
                        // (single-quoted) state.
                        self.get_current_doctype().public_ident = Some(String::new());
                        self.switch_to(TokenizerState::DOCTYPEPublicIdentifierSinglequotedState);
                    }
                    Some('>') => {
                        // This is a missing-doctype-public-identifier parse error. Set the current
                        // DOCTYPE token's force-quirks flag to on. Switch to the data state. Emit
                        // the current DOCTYPE token.
                        self.get_current_doctype().force_quirks = true;
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    Some(_) => {
                        // This is a missing-quote-before-doctype-public-identifier parse error.
                        // Set the current DOCTYPE token's force-quirks flag to on. Reconsume in
                        // the bogus DOCTYPE state.
                        self.get_current_doctype().force_quirks = true;
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::BogusDOCTYPEState);
                    }
                    None => {
                        // This is an eof-in-doctype parse error. Set the current DOCTYPE token's
                        // force-quirks flag to on. Emit the current DOCTYPE token. Emit an
                        // end-of-file token.
                        self.get_current_doctype().force_quirks = true;
                        self.emit_current_token();
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::DOCTYPEPublicIdentifierDoublequotedState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('"') => {
                        // Switch to the after DOCTYPE public identifier state.
                        self.switch_to(TokenizerState::AfterDOCTYPEPublicIdentifierState);
                    }
                    Some('\0') => {
                        // This is an unexpected-null-character parse error. Append a U+FFFD
                        // REPLACEMENT CHARACTER character to the current DOCTYPE token's public
                        // identifier.
                        self.get_current_doctype()
                            .public_ident
                            .as_mut()
                            .map(|ident| ident.push(UNICODE_REPLACEMENT));
                    }
                    Some('>') => {
                        // This is an abrupt-doctype-public-identifier parse error. Set the
                        // current DOCTYPE token's force-quirks flag to on. Switch to the data
                        // state. Emit the current DOCTYPE token.
                        self.get_current_doctype().force_quirks = true;
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    Some(c) => {
                        // Append the current input character to the current DOCTYPE token's
                        // public identifier.
                        self.get_current_doctype()
                            .public_ident
                            .as_mut()
                            .map(|ident| ident.push(c));
                    }
                    None => {
                        // This is an eof-in-doctype parse error. Set the current DOCTYPE token's
                        // force-quirks flag to on. Emit the current DOCTYPE token. Emit an
                        // end-of-file token.
                        self.get_current_doctype().force_quirks = true;
                        self.emit_current_token();
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::DOCTYPEPublicIdentifierSinglequotedState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('\'') => {
                        // Switch to the after DOCTYPE public identifier state.
                        self.switch_to(TokenizerState::AfterDOCTYPEPublicIdentifierState);
                    }
                    Some('\0') => {
                        // This is an unexpected-null-character parse error. Append a U+FFFD
                        // REPLACEMENT CHARACTER character to the current DOCTYPE token's public
                        // identifier.
                        self.get_current_doctype()
                            .public_ident
                            .as_mut()
                            .map(|ident| ident.push(UNICODE_REPLACEMENT));
                    }
                    Some('>') => {
                        // This is an abrupt-doctype-public-identifier parse error. Set the
                        // current DOCTYPE token's force-quirks flag to on. Switch to the data
                        // state. Emit the current DOCTYPE token.
                        self.get_current_doctype().force_quirks = true;
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    Some(c) => {
                        // Append the current input character to the current DOCTYPE token's public
                        // identifier.
                        self.get_current_doctype()
                            .public_ident
                            .as_mut()
                            .map(|ident| ident.push(c));
                    }
                    None => {
                        // This is an eof-in-doctype parse error. Set the current DOCTYPE token's
                        // force-quirks flag to on. Emit the current DOCTYPE token. Emit an
                        // end-of-file token.
                        self.get_current_doctype().force_quirks = true;
                        self.emit_current_token();
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::AfterDOCTYPEPublicIdentifierState => {
                // Consume the next input character:
                match self.read_next() {
                    //       tab       line feed    form feed     space
                    Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}') => {
                        // Switch to the between DOCTYPE public and system identifiers state.
                        self.switch_to(
                            TokenizerState::BetweenDOCTYPEPublicAndSystemIdentifiersState,
                        );
                    }
                    Some('>') => {
                        // Switch to the data state. Emit the current DOCTYPE token.
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    Some('"') => {
                        // This is a
                        // missing-whitespace-between-doctype-public-and-system-identifiers parse
                        // error. Set the current DOCTYPE token's system identifier to the empty
                        // string (not missing), then switch to the DOCTYPE system identifier
                        // (double-quoted) state.
                        self.get_current_doctype().system_ident = Some(String::new());
                        self.switch_to(TokenizerState::DOCTYPESystemIdentifierDoublequotedState);
                    }
                    Some('\'') => {
                        // This is a
                        // missing-whitespace-between-doctype-public-and-system-identifiers
                        // parse error. Set the current DOCTYPE token's system identifier to
                        // the empty string (not missing), then switch to the DOCTYPE system
                        // identifier (single-quoted) state.
                        self.get_current_doctype().system_ident = Some(String::new());
                        self.switch_to(TokenizerState::DOCTYPESystemIdentifierSinglequotedState);
                    }
                    Some(_) => {
                        // This is a missing-quote-before-doctype-system-identifier parse error.
                        // Set the current DOCTYPE token's force-quirks flag to on. Reconsume in
                        // the bogus DOCTYPE state.
                        self.get_current_doctype().force_quirks = true;
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::BogusDOCTYPEState);
                    }
                    None => {
                        // This is an eof-in-doctype parse error. Set the current DOCTYPE token's
                        // force-quirks flag to on. Emit the current DOCTYPE token. Emit an
                        // end-of-file token.
                        self.get_current_doctype().force_quirks = true;
                        self.emit_current_token();
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::BetweenDOCTYPEPublicAndSystemIdentifiersState => {
                // Consume the next input character:
                match self.read_next() {
                    //       tab       line feed    form feed     space
                    Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}') => {} // Ignore the character.
                    Some('>') => {
                        // Switch to the data state. Emit the current DOCTYPE token.
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    Some('"') => {
                        // Set the current DOCTYPE token's system identifier to the empty string
                        // (not missing), then switch to the DOCTYPE system identifier
                        // (double-quoted) state.
                        self.get_current_doctype().system_ident = Some(String::new());
                        self.switch_to(TokenizerState::DOCTYPESystemIdentifierDoublequotedState);
                    }
                    Some('\'') => {
                        // Set the current DOCTYPE token's system identifier to the empty string
                        // (not missing), then switch to the DOCTYPE system identifier
                        // (single-quoted) state.
                        self.get_current_doctype().system_ident = Some(String::new());
                        self.switch_to(TokenizerState::DOCTYPESystemIdentifierSinglequotedState);
                    }
                    Some(_) => {
                        // This is a missing-quote-before-doctype-system-identifier parse error.
                        // Set the current DOCTYPE token's force-quirks flag to on. Reconsume in
                        // the bogus DOCTYPE state.
                        self.get_current_doctype().force_quirks = true;
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::BogusDOCTYPEState);
                    }
                    None => {
                        // This is an eof-in-doctype parse error. Set the current DOCTYPE token's
                        // force-quirks flag to on. Emit the current DOCTYPE token. Emit an
                        // end-of-file token.
                        self.get_current_doctype().force_quirks = true;
                        self.emit_current_token();
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::AfterDOCTYPESystemKeywordState => {
                // Consume the next input character:
                match self.read_next() {
                    //       tab       line feed    form feed     space
                    Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}') => {
                        // Switch to the before DOCTYPE system identifier state.
                        self.switch_to(TokenizerState::BeforeDOCTYPESystemIdentifierState);
                    }
                    Some('"') => {
                        // This is a missing-whitespace-after-doctype-system-keyword parse error.
                        // Set the current DOCTYPE token's system identifier to the empty string
                        // (not missing), then switch to the DOCTYPE system identifier
                        // (double-quoted) state.
                        self.get_current_doctype().system_ident = Some(String::new());
                        self.switch_to(TokenizerState::DOCTYPESystemIdentifierDoublequotedState);
                    }
                    Some('\'') => {
                        // This is a missing-whitespace-after-doctype-system-keyword parse error.
                        // Set the current DOCTYPE token's system identifier to the empty string
                        // (not missing), then switch to the DOCTYPE system identifier
                        // (single-quoted) state.
                        self.get_current_doctype().system_ident = Some(String::new());
                        self.switch_to(TokenizerState::DOCTYPESystemIdentifierSinglequotedState);
                    }
                    Some('>') => {
                        // This is a missing-doctype-system-identifier parse error. Set the current
                        // DOCTYPE token's force-quirks flag to on. Switch to the data state. Emit
                        // the current DOCTYPE token.
                        self.get_current_doctype().force_quirks = true;
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    Some(_) => {
                        // This is a missing-quote-before-doctype-system-identifier parse error.
                        // Set the current DOCTYPE token's force-quirks flag to on. Reconsume in
                        // the bogus DOCTYPE state.
                        self.get_current_doctype().force_quirks = true;
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::BogusDOCTYPEState);
                    }
                    None => {
                        // This is an eof-in-doctype parse error. Set the current DOCTYPE token's
                        // force-quirks flag to on. Emit the current DOCTYPE token. Emit an
                        // end-of-file token.
                        self.get_current_doctype().force_quirks = true;
                        self.emit_current_token();
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::BeforeDOCTYPESystemIdentifierState => {
                // Consume the next input character:
                match self.read_next() {
                    //       tab       line feed    form feed     space
                    Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}') => {} //     Ignore the character.
                    Some('"') => {
                        // Set the current DOCTYPE token's system identifier to the empty string
                        // (not missing), then switch to the DOCTYPE system identifier
                        // (double-quoted) state.
                        self.get_current_doctype().system_ident = Some(String::new());
                        self.switch_to(TokenizerState::DOCTYPESystemIdentifierDoublequotedState);
                    }
                    Some('\'') => {
                        // Set the current DOCTYPE token's system identifier to the empty string
                        // (not missing), then switch to the DOCTYPE system identifier
                        // (single-quoted) state.
                        self.get_current_doctype().system_ident = Some(String::new());
                        self.switch_to(TokenizerState::DOCTYPESystemIdentifierSinglequotedState);
                    }
                    Some('>') => {
                        // This is a missing-doctype-system-identifier parse error. Set the
                        // current DOCTYPE token's force-quirks flag to on. Switch to the data
                        // state. Emit the current DOCTYPE token.
                        self.get_current_doctype().force_quirks = true;
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    Some(_) => {
                        // This is a missing-quote-before-doctype-system-identifier parse error.
                        // Set the current DOCTYPE token's force-quirks flag to on. Reconsume in
                        // the bogus DOCTYPE state.
                        self.get_current_doctype().force_quirks = true;
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::BogusDOCTYPEState);
                    }
                    None => {
                        // This is an eof-in-doctype parse error. Set the current DOCTYPE token's
                        // force-quirks flag to on. Emit the current DOCTYPE token. Emit an
                        // end-of-file token.
                        self.get_current_doctype().force_quirks = true;
                        self.emit_current_token();
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::DOCTYPESystemIdentifierDoublequotedState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('"') => {
                        // Switch to the after DOCTYPE system identifier state.
                        self.switch_to(TokenizerState::AfterDOCTYPESystemIdentifierState);
                    }
                    Some('\0') => {
                        // This is an unexpected-null-character parse error. Append a U+FFFD
                        // REPLACEMENT CHARACTER character to the current DOCTYPE token's system
                        // identifier.
                        self.get_current_doctype()
                            .system_ident
                            .as_mut()
                            .map(|ident| ident.push(UNICODE_REPLACEMENT));
                    }
                    Some('>') => {
                        // This is an abrupt-doctype-system-identifier parse error. Set the current
                        // DOCTYPE token's force-quirks flag to on. Switch to the data state. Emit
                        // the current DOCTYPE token.
                        self.get_current_doctype().force_quirks = true;
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    Some(c) => {
                        // Append the current input character to the current DOCTYPE token's system
                        // identifier.
                        self.get_current_doctype()
                            .system_ident
                            .as_mut()
                            .map(|ident| ident.push(c));
                    }
                    None => {
                        // This is an eof-in-doctype parse error. Set the current DOCTYPE token's
                        // force-quirks flag to on. Emit the current DOCTYPE token. Emit an
                        // end-of-file token.
                        self.get_current_doctype().force_quirks = true;
                        self.emit_current_token();
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::DOCTYPESystemIdentifierSinglequotedState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('\'') => {
                        // Switch to the after DOCTYPE system identifier state.
                        self.switch_to(TokenizerState::AfterDOCTYPESystemIdentifierState);
                    }
                    Some('\0') => {
                        // This is an unexpected-null-character parse error. Append a U+FFFD
                        // REPLACEMENT CHARACTER character to the current DOCTYPE token's system
                        // identifier.
                        self.get_current_doctype()
                            .system_ident
                            .as_mut()
                            .map(|ident| ident.push(UNICODE_REPLACEMENT));
                    }
                    Some('>') => {
                        // This is an abrupt-doctype-system-identifier parse error. Set the current
                        // DOCTYPE token's force-quirks flag to on. Switch to the data state. Emit
                        // the current DOCTYPE token.
                        self.get_current_doctype().force_quirks = true;
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    Some(c) => {
                        // Append the current input character to the current DOCTYPE token's system
                        // identifier.
                        self.get_current_doctype()
                            .system_ident
                            .as_mut()
                            .map(|ident| ident.push(c));
                    }
                    None => {
                        // This is an eof-in-doctype parse error. Set the current DOCTYPE token's
                        // force-quirks flag to on. Emit the current DOCTYPE token. Emit an
                        // end-of-file token.
                        self.get_current_doctype().force_quirks = true;
                        self.emit_current_token();
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::AfterDOCTYPESystemIdentifierState => {
                // Consume the next input character:
                match self.read_next() {
                    //       tab       line feed    form feed     space
                    Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}') => {} // Ignore the character.
                    Some('>') => {
                        // Switch to the data state. Emit the current DOCTYPE token.
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    Some(_) => {
                        // This is an unexpected-character-after-doctype-system-identifier parse
                        // error. Reconsume in the bogus DOCTYPE state. (This does not set the
                        // current DOCTYPE token's force-quirks flag to on.)
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::BogusDOCTYPEState);
                    }
                    None => {
                        // This is an eof-in-doctype parse error. Set the current DOCTYPE
                        // token's force-quirks flag to on. Emit the current DOCTYPE token.
                        // Emit an end-of-file token.
                        self.get_current_doctype().force_quirks = true;
                        self.emit_current_token();
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::BogusDOCTYPEState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('>') => {
                        // Switch to the data state. Emit the DOCTYPE token.
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    }
                    Some('\0') => {
                        // This is an unexpected-null-character parse error. Ignore the character.
                    }
                    Some(_) => {} // Ignore the character.
                    None => {
                        // Emit the DOCTYPE token. Emit an end-of-file token.
                        self.emit_current_token();
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::CDATASectionState => {
                // Consume the next input character:
                match self.read_next() {
                    Some(']') => {
                        // Switch to the CDATA section bracket state.
                        self.switch_to(TokenizerState::CDATASectionBracketState);
                    }
                    Some(c) => {
                        // Emit the current input character as a character token.
                        self.emit(Token::Character(c));
                    }
                    None => {
                        // This is an eof-in-cdata parse error. Emit an end-of-file token.
                        self.emit(Token::EOF);
                    }
                }
            }
            TokenizerState::CDATASectionBracketState => {
                // Consume the next input character:
                match self.read_next() {
                    Some(']') => {
                        // Switch to the CDATA section end state.
                        self.switch_to(TokenizerState::CDATASectionEndState);
                    }
                    _ => {
                        // Emit a U+005D RIGHT SQUARE BRACKET character token. Reconsume in the
                        // CDATA section state.
                        self.emit(Token::Character(']'));
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::CDATASectionState);
                    }
                }
            }
            TokenizerState::CDATASectionEndState => {
                // Consume the next input character:
                match self.read_next() {
                    Some(']') => {
                        // Emit a U+005D RIGHT SQUARE BRACKET character token.
                        self.emit(Token::Character(']'));
                    }
                    Some('>') => {
                        // Switch to the data state.
                        self.switch_to(TokenizerState::DataState);
                    }
                    _ => {
                        // Emit two U+005D RIGHT SQUARE BRACKET character tokens. Reconsume in the
                        // CDATA section state.
                        self.emit(Token::Character(']'));
                        self.emit(Token::Character(']'));
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::CDATASectionState);
                    }
                }
            }
            TokenizerState::CharacterReferenceState => {
                // Set the temporary buffer to the empty string. Append a U+0026 AMPERSAND (&)
                // character to the temporary buffer.
                self.buffer = Some("&".to_string());

                // Consume the next input character:
                match self.read_next() {
                    Some('a'..='z' | 'A'..='Z' | '0'..='9') => {
                        // Reconsume in the named character reference state.
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::NamedCharacterReferenceState);
                    }
                    Some('#') => {
                        // Append the current input character to the temporary buffer. Switch to
                        // the numeric character reference state.
                        self.add_to_buffer('#');
                        self.switch_to(TokenizerState::NumericCharacterReferenceState);
                    }
                    _ => {
                        // Flush code points consumed as a character reference. Reconsume in the
                        // return state.

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
                        // If the character reference was consumed as part of an attribute, and
                        // the last character matched is not a U+003B SEMICOLON character (;),
                        // and the next input character is either a U+003D EQUALS SIGN
                        // character (=) or an ASCII alphanumeric, then, for historical
                        // reasons, flush code points consumed as a character reference and
                        // switch to the return state.
                        //
                        // Otherwise:
                        // If the last character matched is not a U+003B SEMICOLON
                        // character (;), then this is a
                        // missing-semicolon-after-character-reference parse error.
                        //
                        // Set the temporary buffer to the empty string.
                        // Append one or two characters corresponding to
                        // the character reference name (as given by the
                        // second column of the named character references
                        // table) to the temporary buffer.
                        // Flush code points consumed as a
                        // character reference. Switch to the
                        // return state.
                        _ = unicode_val;
                        todo!();
                    }
                    None => {
                        // Flush code points consumed as a character reference. Switch to the
                        // ambiguous ampersand state.
                        self.switch_to(TokenizerState::AmbiguousAmpersandState);
                        todo!("flush code points consumed as character reference");
                    }
                }
            }
            TokenizerState::AmbiguousAmpersandState => {
                // Consume the next input character:
                match self.read_next() {
                    Some(c @ ('a'..='z' | 'A'..='Z' | '0'..='9')) => {
                        let was_consumed_as_part_of_attr = false; // TODO
                                                                  // If the character reference was consumed as part of an attribute,
                        if was_consumed_as_part_of_attr {
                            // then append the current input character to the current attribute's
                            // value.
                            self.get_current_tag().add_to_attr_value(c);
                        } else {
                            // Otherwise, emit the current input character as a character
                            // token.
                            self.emit(Token::Character(c));
                        }
                        todo!();
                    }
                    Some(';') => {
                        // This is an unknown-named-character-reference parse error. Reconsume in
                        // the return state.
                        self.ptr -= 1;
                        self.switch_to(self.return_state.unwrap());
                    }
                    _ => {
                        // Reconsume in the return state.
                        self.ptr -= 1;
                        self.switch_to(self.return_state.unwrap());
                    }
                }
            }
            TokenizerState::NumericCharacterReferenceState => {
                // Set the character reference code to zero (0).
                self.character_reference_code = 0;

                // Consume the next input character:
                match self.read_next() {
                    Some(c @ ('X' | 'x')) => {
                        // Append the current input character to the temporary buffer. Switch to
                        // the hexadecimal character reference start state.
                        self.add_to_buffer(c);
                        self.switch_to(TokenizerState::HexadecimalCharacterReferenceStartState);
                    }
                    _ => {
                        // Reconsume in the decimal character reference start state.
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::DecimalCharacterReferenceStartState);
                    }
                }
            }
            TokenizerState::HexadecimalCharacterReferenceStartState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('0'..='9' | 'a'..='f' | 'A'..='F') => {
                        // Reconsume in the hexadecimal character reference state.
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::HexadecimalCharacterReferenceState);
                    }
                    _ => {
                        // This is an absence-of-digits-in-numeric-character-reference parse error.
                        // Flush code points consumed as a character reference. Reconsume in the
                        // return state.
                        self.ptr -= 1;
                        self.switch_to(self.return_state.unwrap());
                        todo!("Flush code points consumed as a character reference.");
                    }
                }
            }
            TokenizerState::DecimalCharacterReferenceStartState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('0'..='9') => {
                        // Reconsume in the decimal character reference state.
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::DecimalCharacterReferenceState);
                    }
                    _ => {
                        // This is an absence-of-digits-in-numeric-character-reference parse
                        // error. Flush code points consumed as a character reference.
                        // Reconsume in the return state.
                        self.ptr -= 1;
                        self.switch_to(self.return_state.unwrap());
                        todo!("Flush code points consumed as a character reference.");
                    }
                }
            }
            TokenizerState::HexadecimalCharacterReferenceState => {
                // Consume the next input character:
                match self.read_next() {
                    Some(c @ ('0'..='9' | 'a'..='f' | 'A'..='F')) => {
                        // Multiply the character reference code by 16. Add a numeric version of
                        // the current input character to the character reference code.
                        self.character_reference_code *= 16;
                        self.character_reference_code += c.to_digit(16).unwrap();
                    }
                    Some(';') => {
                        // Switch to the numeric character reference end state.
                        self.switch_to(TokenizerState::NumericCharacterReferenceEndState);
                    }
                    _ => {
                        // This is a missing-semicolon-after-character-reference parse error.
                        // Reconsume in the numeric character reference end state.
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::NumericCharacterReferenceEndState);
                    }
                }
            }
            TokenizerState::DecimalCharacterReferenceState => {
                // Consume the next input character:
                match self.read_next() {
                    Some(c @ '0'..='9') => {
                        // Multiply the character reference code by 10. Add a numeric version of
                        // the current input character (subtract 0x0030 from the character's code
                        // point) to the character reference code.
                        self.character_reference_code *= 10;
                        self.character_reference_code += c.to_digit(10).unwrap();
                    }
                    Some(';') => {
                        // Switch to the numeric character reference end state.
                        self.switch_to(TokenizerState::NumericCharacterReferenceEndState);
                    }
                    _ => {
                        // This is a missing-semicolon-after-character-reference parse error.
                        // Reconsume in the numeric character reference end state.
                        self.ptr -= 1;
                        self.switch_to(TokenizerState::NumericCharacterReferenceEndState);
                    }
                }
            }
            TokenizerState::NumericCharacterReferenceEndState => {
                // Check the character reference code:
                match self.character_reference_code {
                    0x00 => {
                        // This is a null-character-reference parse error. Set the character
                        // reference code to 0xFFFD.
                        self.character_reference_code = 0xFFFD;
                    }
                    0x110000.. => {
                        // This is a character-reference-outside-unicode-range parse error. Set the
                        // character reference code to 0xFFFD.
                        self.character_reference_code = 0xFFFD;
                    }
                    0xD800..=0xDFFF => {
                        // This is a surrogate-character-reference parse error. Set the character
                        // reference code to 0xFFFD.
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
                        // This is a noncharacter-character-reference parse error.
                        //
                        // (the spec doesn't seem to specify how those should be handled)
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
            log::info!("{:?}", first_token);
            if let Some(Token::EOF) = first_token {
                self.done = true;
            }
            first_token
        }
    }
}
