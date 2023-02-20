//! Implements a [HTML Tokenizer](https://html.spec.whatwg.org/multipage/parsing.html#tokenization)

use super::character_reference::match_reference;
use std::collections::VecDeque;

const UNICODE_REPLACEMENT: char = '\u{FFFD}';

const TAB: char = '\u{0009}';
const LINE_FEED: char = '\u{000A}';
const FORM_FEED: char = '\u{000C}';
const SPACE: char = '\u{0020}';

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

    fn reconsume_in(&mut self, new_state: TokenizerState) {
        self.ptr -= 1;
        self.state = new_state;
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
            },
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
        let c = self.source.chars().nth(self.ptr);
        self.ptr += 1;
        c
    }

    pub fn step(&mut self) {
        match self.state {
            // https://html.spec.whatwg.org/multipage/parsing.html#data-state
            TokenizerState::DataState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('&') => {
                        // Set the return state to the data state. Switch to the character
                        // reference state.
                        self.return_state = Some(TokenizerState::DataState);
                        self.switch_to(TokenizerState::CharacterReferenceState);
                    },
                    Some('<') => {
                        // Switch to the tag open state.
                        self.switch_to(TokenizerState::TagOpenState);
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        // Emit the current input character as a character token.
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    },
                    Some(c) => {
                        // Emit the current input character as a character token.
                        self.emit(Token::Character(c));
                    },
                    None => {
                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#rcdata-state
            TokenizerState::RCDATAState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('&') => {
                        // Set the return state to the RCDATA state. Switch to the character
                        // reference state.
                        self.return_state = Some(TokenizerState::RCDATAState);
                        self.switch_to(TokenizerState::CharacterReferenceState);
                    },
                    Some('<') => {
                        // Switch to the RCDATA less-than sign state.
                        self.switch_to(TokenizerState::RCDATALessThanSignState);
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        // Emit the current input character as a character token.
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    },
                    Some(c) => {
                        // Emit the current input character as a character token.
                        self.emit(Token::Character(c));
                    },
                    None => {
                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#rawtext-state
            TokenizerState::RAWTEXTState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('<') => {
                        // Switch to the RAWTEXT less-than sign state.
                        self.switch_to(TokenizerState::RAWTEXTLessThanSignState);
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        // Emit the current input character as a character token.
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    },
                    Some(c) => {
                        // Emit the current input character as a character token.
                        self.emit(Token::Character(c));
                    },
                    None => {
                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-state
            TokenizerState::ScriptDataState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('<') => {
                        // Switch to the script data less-than sign state.
                        self.switch_to(TokenizerState::ScriptDataLessThanSignState);
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        // Emit the current input character as a character token.
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    },
                    Some(c) => {
                        // Emit the current input character as a character token.
                        self.emit(Token::Character(c));
                    },
                    None => {
                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#plaintext-state
            TokenizerState::PLAINTEXTState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        // Emit the current input character as a character token.
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    },
                    Some(c) => {
                        // Emit the current input character as a character token.
                        self.emit(Token::Character(c));
                    },
                    None => {
                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#tag-open-state
            TokenizerState::TagOpenState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('!') => {
                        // Switch to the markup declaration open state.
                        self.switch_to(TokenizerState::MarkupDeclarationOpenState);
                    },
                    Some('/') => {
                        //     Switch to the end tag open state.
                        self.switch_to(TokenizerState::EndTagOpenState);
                    },
                    Some('a'..='z' | 'A'..='Z') => {
                        // Create a new start tag token, set its tag name to the empty string.
                        self.current_token = Some(Token::Tag(TagData::default_open()));

                        // Reconsume in the tag name state.
                        self.reconsume_in(TokenizerState::TagNameState);
                    },
                    Some('?') => {
                        // This is an unexpected-question-mark-instead-of-tag-name parse error.
                        // Create a comment token whose data is the empty string.
                        self.current_token = Some(Token::Comment(String::default()));

                        // Reconsume in the bogus comment state.
                        self.reconsume_in(TokenizerState::BogusCommentState);
                    },
                    Some(_) => {
                        // This is an invalid-first-character-of-tag-name parse error.
                        // Emit a U+003C LESS-THAN SIGN character token.
                        self.emit(Token::Character('<'));

                        // Reconsume in the data state.
                        self.reconsume_in(TokenizerState::DataState);
                    },
                    None => {
                        // This is an eof-before-tag-name parse error.
                        // Emit a U+003C LESS-THAN SIGN
                        // character token and an end-of-file token.
                        self.emit(Token::Character('<'));
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#end-tag-open-state
            TokenizerState::EndTagOpenState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('a'..='z' | 'A'..='Z') => {
                        // Create a new end tag token, set its tag name to the empty string.
                        self.current_token = Some(Token::Tag(TagData::default_close()));

                        // Reconsume in the tag name state.
                        self.reconsume_in(TokenizerState::TagNameState);
                    },
                    Some('>') => {
                        // This is a missing-end-tag-name parse error.
                        // Switch to the data state.
                        self.switch_to(TokenizerState::DataState);
                    },
                    Some(_) => {
                        // This is an invalid-first-character-of-tag-name parse error.
                        // Create a comment token whose data is the empty string.
                        self.current_token = Some(Token::Comment(String::default()));

                        // Reconsume in the bogus comment state.
                        self.reconsume_in(TokenizerState::BogusCommentState);
                    },
                    None => {
                        // This is an eof-before-tag-name parse error.
                        // Emit a U+003C LESS-THAN SIGN character token,
                        // a U+002F SOLIDUS character token and an end-of-file
                        // token.
                        self.emit(Token::Character('<'));
                        self.emit(Token::Character('/'));
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#tag-name-state
            TokenizerState::TagNameState => {
                // Consume the next input character:
                match self.read_next() {
                    Some(TAB | LINE_FEED | FORM_FEED | SPACE) => {
                        // Switch to the before attribute name state.
                        self.switch_to(TokenizerState::BeforeAttributeNameState);
                    },
                    Some('/') => {
                        // Switch to the self-closing start tag state.
                        self.switch_to(TokenizerState::SelfClosingStartTagState);
                    },
                    Some('>') => {
                        // Switch to the data state. Emit the current tag token.
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    },
                    Some(mut c @ 'A'..='Z') => {
                        // Append the lowercase version of the current input character (add
                        // 0x0020 to the character's code point) to the current tag token's tag
                        // name.
                        c.make_ascii_lowercase();
                        self.get_current_tag().name.push(c);
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error. Append a U+FFFD
                        // REPLACEMENT CHARACTER character to the current tag token's tag name.
                        self.get_current_tag().name.push(UNICODE_REPLACEMENT);
                    },
                    Some(c) => {
                        // Append the current input character to the current tag token's tag name.
                        self.get_current_tag().name.push(c);
                    },
                    None => {
                        // This is an eof-in-tag parse error. Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#rcdata-less-than-sign-state
            TokenizerState::RCDATALessThanSignState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('/') => {
                        // Set the temporary buffer to the empty string. Switch to the RCDATA end
                        // tag open state.
                        self.buffer = Some(String::new());
                        self.switch_to(TokenizerState::RCDATAEndTagOpenState);
                    },
                    _ => {
                        // Emit a U+003C LESS-THAN SIGN character token.
                        self.emit(Token::Character('<'));

                        // Reconsume in the RCDATA state.
                        self.reconsume_in(TokenizerState::RCDATAState);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#rcdata-end-tag-open-state
            TokenizerState::RCDATAEndTagOpenState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('a'..='z' | 'A'..='Z') => {
                        // Create a new end tag token, set its tag name to the empty string.
                        self.current_token = Some(Token::Tag(TagData::default_close()));

                        // Reconsume in the RCDATA end tag name state.
                        self.reconsume_in(TokenizerState::RCDATAEndTagNameState);
                    },
                    _ => {
                        // Emit a U+003C LESS-THAN SIGN character token and a U+002F SOLIDUS
                        // character token.
                        self.emit(Token::Character('<'));
                        self.emit(Token::Character('/'));

                        // Reconsume in the RCDATA state.
                        self.reconsume_in(TokenizerState::RCDATAState);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#rcdata-end-tag-name-state
            TokenizerState::RCDATAEndTagNameState => {
                // Consume the next input character:
                match (self.read_next(), self.is_appropriate_end_token()) {
                    (Some(TAB | LINE_FEED | FORM_FEED | SPACE), true) => {
                        // Switch to the before attribute name state.
                        self.switch_to(TokenizerState::BeforeAttributeNameState);
                    },
                    (Some('/'), true) => {
                        // Switch to the self-closing start tag state.
                        self.switch_to(TokenizerState::SelfClosingStartTagState);
                    },
                    (Some('>'), true) => {
                        // Switch to the data state and emit the current tag token
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    },
                    (Some(mut c @ 'A'..='Z'), _) => {
                        // Append the lowercase version of the current input character (add 0x0020
                        // to the character's code point) to the current tag token's tag name.
                        // Append the current input character to the temporary buffer.
                        self.add_to_buffer(c);
                        c.make_ascii_lowercase();
                        self.get_current_tag().name.push(c);
                    },
                    (Some(c @ 'a'..='z'), _) => {
                        // Append the current input character to the current tag token's tag name.
                        // Append the current input character to the temporary buffer.
                        self.add_to_buffer(c);
                        self.get_current_tag().name.push(c);
                    },
                    _ => {
                        // Emit a U+003C LESS-THAN SIGN character token, a U+002F SOLIDUS character
                        // token, and a character token for each of the characters in the temporary
                        // buffer (in the order they were added to the buffer).
                        self.emit(Token::Character('<'));
                        self.emit(Token::Character('/'));
                        for c in self.buffer.take().unwrap().chars() {
                            self.emit(Token::Character(c));
                        }

                        // Reconsume in the RCDATA state.
                        self.reconsume_in(TokenizerState::RCDATAState);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#rawtext-less-than-sign-state
            TokenizerState::RAWTEXTLessThanSignState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('/') => {
                        // Set the temporary buffer to the empty string.
                        // Switch to the RAWTEXT end tag open state.
                        self.buffer = Some(String::new());
                        self.switch_to(TokenizerState::RAWTEXTEndTagOpenState);
                    },
                    _ => {
                        // Emit a U+003C LESS-THAN SIGN character token.
                        self.emit(Token::Character('<'));

                        // Reconsume in the RAWTEXT state.
                        self.reconsume_in(TokenizerState::RAWTEXTState);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#rawtext-end-tag-open-state
            TokenizerState::RAWTEXTEndTagOpenState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('a'..='z' | 'A'..='Z') => {
                        // Create a new end tag token, set its tag name to the empty string.
                        self.current_token = Some(Token::Tag(TagData::default_close()));

                        // Reconsume in the RAWTEXT end tag name state.
                        self.reconsume_in(TokenizerState::RAWTEXTEndTagNameState);
                    },
                    _ => {
                        // Emit a U+003C LESS-THAN SIGN character token and a U+002F SOLIDUS
                        // character token.
                        self.emit(Token::Character('<'));
                        self.emit(Token::Character('/'));

                        // Reconsume in the RAWTEXT state.
                        self.reconsume_in(TokenizerState::RAWTEXTState);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#rawtext-end-tag-name-state
            TokenizerState::RAWTEXTEndTagNameState => {
                // Consume the next input character:
                match (self.read_next(), self.is_appropriate_end_token()) {
                    (Some(TAB | LINE_FEED | FORM_FEED | SPACE), true) => {
                        // Switch to the before attribute name state
                        self.switch_to(TokenizerState::BeforeAttributeNameState);
                    },
                    (Some('/'), true) => {
                        // Switch to the self-closing start tag state
                        self.switch_to(TokenizerState::SelfClosingStartTagState);
                    },
                    (Some('>'), true) => {
                        // Switch to the data state and emit the current tag token
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    },
                    (Some(c @ 'A'..='Z'), _) => {
                        // Append the lowercase version of the current input character (add 0x0020
                        // to the character's code point) to the current tag token's tag name.
                        self.get_current_tag().name.push(c.to_ascii_lowercase());

                        // Append the current input character to the temporary buffer.
                        self.add_to_buffer(c);
                    },
                    (Some(c @ 'a'..='z'), _) => {
                        // Append the current input character to the current tag token's tag name.
                        // Append the current input character to the temporary buffer.
                        self.add_to_buffer(c);
                        self.get_current_tag().name.push(c);
                    },
                    _ => {
                        // Emit a U+003C LESS-THAN SIGN character token, a U+002F SOLIDUS character
                        // token, and a character token for each of the characters in the temporary
                        // buffer (in the order they were added to the buffer).
                        self.emit(Token::Character('<'));
                        self.emit(Token::Character('/'));
                        for c in self.buffer.take().unwrap().chars() {
                            self.emit(Token::Character(c));
                        }

                        // Reconsume in the RAWTEXT state.
                        self.reconsume_in(TokenizerState::RAWTEXTState);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-less-than-sign-state
            TokenizerState::ScriptDataLessThanSignState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('/') => {
                        // Set the temporary buffer to the empty string.
                        self.buffer = Some(String::new());

                        // Switch to the script data end tag open state.
                        self.switch_to(TokenizerState::ScriptDataEndTagOpenState);
                    },
                    Some('!') => {
                        // Switch to the script data escape start state.
                        self.switch_to(TokenizerState::ScriptDataEscapeStartState);

                        // Emit a U+003C LESS-THAN SIGN character token and a
                        // U+0021 EXCLAMATION MARK character token.
                        self.emit(Token::Character('<'));
                        self.emit(Token::Character('!'));
                    },
                    _ => {
                        // Emit a U+003C LESS-THAN SIGN character token.
                        self.emit(Token::Character('<'));

                        // Reconsume in the script data state.
                        self.reconsume_in(TokenizerState::ScriptDataState);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-end-tag-open-state
            TokenizerState::ScriptDataEndTagOpenState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('a'..='z' | 'A'..='Z') => {
                        // Create a new end tag token, set its tag name to the empty string.
                        self.current_token = Some(Token::Tag(TagData::default_close()));

                        // Reconsume in the script data end tag name state.
                        self.reconsume_in(TokenizerState::ScriptDataEndTagNameState);
                    },
                    _ => {
                        // Emit a U+003C LESS-THAN SIGN character token and a U+002F SOLIDUS
                        // character token.
                        self.emit(Token::Character('<'));
                        self.emit(Token::Character('/'));

                        // Reconsume in the script data state.
                        self.reconsume_in(TokenizerState::ScriptDataState);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-end-tag-name-state
            TokenizerState::ScriptDataEndTagNameState => {
                // Consume the next input character:
                match (self.read_next(), self.is_appropriate_end_token()) {
                    (Some(TAB | LINE_FEED | FORM_FEED | SPACE), true) => {
                        // Switch to the before attribute name state
                        self.switch_to(TokenizerState::BeforeAttributeNameState);
                    },
                    (Some('/'), true) => {
                        // Switch to the self-closing start tag state
                        self.switch_to(TokenizerState::SelfClosingStartTagState);
                    },
                    (Some('>'), true) => {
                        // Switch to the data state and emit the current tag token
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    },
                    (Some(c @ 'A'..='Z'), _) => {
                        // Append the lowercase version of the current input character (add
                        // 0x0020 to the character's code point) to the current tag token's tag
                        // name.
                        self.get_current_tag().name.push(c.to_ascii_lowercase());

                        // Append the current input character to the temporary buffer.
                        self.add_to_buffer(c);
                    },
                    (Some(c @ 'a'..='z'), _) => {
                        // Append the current input character to the current tag token's tag name.
                        self.get_current_tag().name.push(c);

                        // Append the current input character to the temporary buffer.
                        self.add_to_buffer(c);
                    },
                    _ => {
                        // Emit a U+003C LESS-THAN SIGN character token, a U+002F SOLIDUS character
                        // token, and a character token for each of the characters in the temporary
                        // buffer (in the order they were added to the buffer).
                        self.emit(Token::Character('<'));
                        self.emit(Token::Character('/'));
                        for c in self.buffer.take().unwrap().chars() {
                            self.emit(Token::Character(c));
                        }

                        // Reconsume in the script data state.
                        self.reconsume_in(TokenizerState::ScriptDataState);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escape-start-state
            TokenizerState::ScriptDataEscapeStartState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('-') => {
                        // Switch to the script data escape start dash state.
                        self.switch_to(TokenizerState::ScriptDataEscapeStartDashState);

                        // Emit a U+002D HYPHEN-MINUS character token.
                        self.emit(Token::Character('-'));
                    },
                    _ => {
                        // Reconsume in the script data state.
                        self.reconsume_in(TokenizerState::ScriptDataState);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escape-start-dash-state
            TokenizerState::ScriptDataEscapeStartDashState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('-') => {
                        // Switch to the script data escaped dash dash state.
                        self.switch_to(TokenizerState::ScriptDataEscapedDashDashState);

                        // Emit a U+002D HYPHEN-MINUS character token.
                        self.emit(Token::Character('-'));
                    },
                    _ => {
                        // Reconsume in the script data state.
                        self.reconsume_in(TokenizerState::ScriptDataState);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escaped-state
            TokenizerState::ScriptDataEscapedState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('-') => {
                        // Switch to the script data escaped dash state.
                        self.switch_to(TokenizerState::ScriptDataEscapedDashState);

                        // Emit a U+002D HYPHEN-MINUS character token.
                        self.emit(Token::Character('-'));
                    },
                    Some('<') => {
                        // Switch to the script data escaped less-than sign state.
                        self.switch_to(TokenizerState::ScriptDataEscapedLessThanSignState);
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        // Emit a U+FFFD REPLACEMENT CHARACTER character token.
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    },
                    Some(c) => {
                        // Emit the current input character as a character token.
                        self.emit(Token::Character(c));
                    },
                    None => {
                        // This is an eof-in-script-html-comment-like-text parse error. Emit an
                        // end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escaped-dash-state
            TokenizerState::ScriptDataEscapedDashState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('-') => {
                        // Switch to the script data escaped dash dash state.
                        self.switch_to(TokenizerState::ScriptDataEscapedDashDashState);

                        // Emit a U+002D HYPHEN-MINUS character token.
                        self.emit(Token::Character('-'));
                    },
                    Some('<') => {
                        // Switch to the script data escaped less-than sign state.
                        self.switch_to(TokenizerState::ScriptDataEscapedLessThanSignState);
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        // Emit a U+FFFD REPLACEMENT CHARACTER character token.
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    },
                    Some(c) => {
                        // Switch to the script data escaped state.
                        self.switch_to(TokenizerState::ScriptDataEscapedState);

                        // Emit the current input character as a character token.
                        self.emit(Token::Character(c));
                    },
                    None => {
                        // This is an eof-in-script-html-comment-like-text parse error.
                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escaped-dash-dash-state
            TokenizerState::ScriptDataEscapedDashDashState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('-') => {
                        // Emit a U+002D HYPHEN-MINUS character token.
                        self.emit(Token::Character('-'));
                    },
                    Some('<') => {
                        // Switch to the script data escaped less-than sign state.
                        self.switch_to(TokenizerState::ScriptDataEscapedLessThanSignState);
                    },
                    Some('>') => {
                        // Switch to the script data state.
                        self.switch_to(TokenizerState::ScriptDataState);

                        // Emit a U+003E GREATER-THAN SIGN character token.
                        self.emit(Token::Character('>'));
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        // Emit a U+FFFD REPLACEMENT CHARACTER character token.
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    },
                    Some(c) => {
                        // Switch to the script data escaped state.
                        self.switch_to(TokenizerState::ScriptDataEscapedState);

                        // Emit the current input character as a character token.
                        self.emit(Token::Character(c));
                    },
                    None => {
                        // This is an eof-in-script-html-comment-like-text parse error. Emit an
                        // end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escaped-less-than-sign-state
            TokenizerState::ScriptDataEscapedLessThanSignState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('/') => {
                        // Set the temporary buffer to the empty string.
                        self.buffer = Some(String::new());

                        // Switch to the script data escaped end tag open state.
                        self.switch_to(TokenizerState::ScriptDataEscapedEndTagOpenState);
                    },
                    Some('a'..='z' | 'A'..='Z') => {
                        // Set the temporary buffer to the empty string. Emit a U+003C LESS-THAN
                        // SIGN character token.
                        self.buffer = Some(String::new());
                        self.emit(Token::Character('<'));

                        // Reconsume in the script data double escape start state.
                        self.reconsume_in(TokenizerState::ScriptDataDoubleEscapeStartState);
                    },
                    _ => {
                        // Emit a U+003C LESS-THAN SIGN character token.
                        self.emit(Token::Character('<'));

                        // Reconsume in the script data escaped state.
                        self.reconsume_in(TokenizerState::ScriptDataEscapedState);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escaped-end-tag-open-state
            TokenizerState::ScriptDataEscapedEndTagOpenState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('a'..='z' | 'A'..='Z') => {
                        // Create a new end tag token, set its tag name to the empty string.
                        self.current_token = Some(Token::Tag(TagData::default_close()));

                        // Reconsume in the script data escaped end tag name state.
                        self.reconsume_in(TokenizerState::ScriptDataEscapedEndTagNameState);
                    },
                    _ => {
                        // Emit a U+003C LESS-THAN SIGN character token and a U+002F SOLIDUS
                        // character token.
                        self.emit(Token::Character('<'));
                        self.emit(Token::Character('/'));

                        // Reconsume in the script data escaped state.
                        self.reconsume_in(TokenizerState::ScriptDataEscapedState);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escaped-end-tag-name-state
            TokenizerState::ScriptDataEscapedEndTagNameState => {
                // Consume the next input character:
                match (self.read_next(), self.is_appropriate_end_token()) {
                    (Some(TAB | LINE_FEED | FORM_FEED | SPACE), true) => {
                        // Switch to the before attribute name state.
                        self.switch_to(TokenizerState::BeforeAttributeNameState);
                    },
                    (Some('/'), true) => {
                        // Switch to the self-closing start tag state.
                        self.switch_to(TokenizerState::SelfClosingStartTagState);
                    },
                    (Some('>'), true) => {
                        // Switch to the data state and emit the current tag token.
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    },
                    (Some(c @ 'A'..='Z'), _) => {
                        // Append the lowercase version of the current input character (add 0x0020
                        // to the character's code point) to the current tag token's tag name.
                        self.get_current_tag().name.push(c.to_ascii_lowercase());

                        // Append the current input character to the temporary buffer.
                        self.add_to_buffer(c);
                    },
                    (Some(c @ 'a'..='z'), _) => {
                        // Append the current input character to the current tag token's tag name.
                        self.get_current_tag().name.push(c);

                        // Append the current input character to the temporary buffer.
                        self.add_to_buffer(c);
                    },
                    _ => {
                        // Emit a U+003C LESS-THAN SIGN character token, a U+002F SOLIDUS character
                        // token, and a character token for each of the characters in the temporary
                        // buffer (in the order they were added to the buffer).
                        self.emit(Token::Character('<'));
                        self.emit(Token::Character('/'));
                        for c in self.buffer.take().unwrap().chars() {
                            self.emit(Token::Character(c));
                        }

                        // Reconsume in the script data escaped state.
                        self.reconsume_in(TokenizerState::ScriptDataEscapedState);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-double-escape-start-state
            TokenizerState::ScriptDataDoubleEscapeStartState => {
                // Consume the next input character:
                match self.read_next() {
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
                    },
                    Some(c @ 'A'..='Z') => {
                        // Append the lowercase version of the current input character (add 0x0020
                        // to the character's code point) to the temporary buffer.
                        self.add_to_buffer(c.to_ascii_lowercase());

                        // Emit the current input character as a character token.
                        self.emit(Token::Character(c));
                    },
                    Some(c @ 'a'..='z') => {
                        // Append the current input character to the temporary buffer.
                        self.add_to_buffer(c);

                        // Emit the current input character as a character token.
                        self.emit(Token::Character(c));
                    },
                    _ => {
                        // Reconsume in the script data escaped state.
                        self.reconsume_in(TokenizerState::ScriptDataEscapedState);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-double-escaped-state
            TokenizerState::ScriptDataDoubleEscapedState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('-') => {
                        // Switch to the script data double escaped dash state.
                        self.switch_to(TokenizerState::ScriptDataDoubleEscapedDashState);

                        // Emit a U+002D HYPHEN-MINUS character token.
                        self.emit(Token::Character('-'));
                    },
                    Some('<') => {
                        // Switch to the script data double escaped less-than sign state.
                        self.switch_to(TokenizerState::ScriptDataDoubleEscapedLessThanSignState);

                        // Emit a U+003C LESS-THAN SIGN character token.
                        self.emit(Token::Character('<'));
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        // Emit a U+FFFD REPLACEMENT CHARACTER character token.
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    },
                    Some(c) => {
                        // Emit the current input character as a character token.
                        self.emit(Token::Character(c));
                    },
                    None => {
                        // This is an eof-in-script-html-comment-like-text parse error.
                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-double-escaped-dash-state
            TokenizerState::ScriptDataDoubleEscapedDashState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('-') => {
                        // Switch to the script data double escaped dash dash state.
                        self.switch_to(TokenizerState::ScriptDataDoubleEscapedDashDashState);

                        // Emit a U+002D HYPHEN-MINUS character token.
                        self.emit(Token::Character('-'));
                    },
                    Some('<') => {
                        // This is an unexpected-null-character parse error.
                        // Switch to the script data double escaped state.
                        self.switch_to(TokenizerState::ScriptDataDoubleEscapedState);

                        // Emit a U+FFFD REPLACEMENT CHARACTER character token.
                        self.emit(Token::Character('<'));
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        // Switch to the script data double escaped state.
                        self.switch_to(TokenizerState::ScriptDataDoubleEscapedState);

                        // Emit a U+FFFD REPLACEMENT CHARACTER character token.
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    },
                    Some(c) => {
                        // Switch to the script data double escaped state.
                        self.switch_to(TokenizerState::ScriptDataDoubleEscapedState);

                        // Emit the current input character as a character token.
                        self.emit(Token::Character(c));
                    },
                    None => {
                        // This is an eof-in-script-html-comment-like-text parse error.
                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-double-escaped-dash-dash-state
            TokenizerState::ScriptDataDoubleEscapedDashDashState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('-') => {
                        // Emit a U+002D HYPHEN-MINUS character token.
                        self.emit(Token::Character('-'));
                    },
                    Some('<') => {
                        // Switch to the script data double escaped less-than sign state.
                        self.switch_to(TokenizerState::ScriptDataDoubleEscapedLessThanSignState);

                        // Emit a U+003C LESS-THAN SIGN character token.
                        self.emit(Token::Character('<'));
                    },
                    Some('>') => {
                        // Switch to the script data state.
                        self.switch_to(TokenizerState::ScriptDataState);

                        // Emit a U+003E GREATER-THAN SIGN character token.
                        self.emit(Token::Character('>'));
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        // Switch to the script data double escaped state.
                        self.switch_to(TokenizerState::ScriptDataDoubleEscapedState);

                        // Emit a U+FFFD REPLACEMENT CHARACTER character token.
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    },
                    Some(c) => {
                        // Switch to the script data double escaped state.
                        self.switch_to(TokenizerState::ScriptDataDoubleEscapedState);

                        // Emit the current input character as a character token.
                        self.emit(Token::Character(c));
                    },
                    None => {
                        // This is an eof-in-script-html-comment-like-text parse error.
                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-double-escaped-less-than-sign-state
            TokenizerState::ScriptDataDoubleEscapedLessThanSignState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('/') => {
                        // Set the temporary buffer to the empty string.
                        self.buffer = Some(String::new());

                        // Switch to the script data double escape end state.
                        self.switch_to(TokenizerState::ScriptDataDoubleEscapeEndState);

                        // Emit a U+002F SOLIDUS character token.
                        self.emit(Token::Character('/'));
                    },
                    _ => {
                        // Reconsume in the script data double escaped state.
                        self.reconsume_in(TokenizerState::ScriptDataDoubleEscapedState);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-double-escape-end-state
            TokenizerState::ScriptDataDoubleEscapeEndState => {
                // Consume the next input character:
                match self.read_next() {
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
                    },
                    Some(c @ 'A'..='Z') => {
                        // Append the lowercase version of the current input character (add
                        // 0x0020 to the character's code point) to the temporary buffer.
                        self.add_to_buffer(c.to_ascii_lowercase());

                        // Emit the current input character as a character token.
                        self.emit(Token::Character(c));
                    },
                    Some(c @ 'a'..='z') => {
                        // Append the current input character to the temporary buffer.
                        self.add_to_buffer(c);

                        // Emit the current input character as a character token.
                        self.emit(Token::Character(c));
                    },
                    _ => {
                        // Reconsume in the script data double escaped state.
                        self.reconsume_in(TokenizerState::ScriptDataDoubleEscapedState);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#before-attribute-name-state
            TokenizerState::BeforeAttributeNameState => {
                // Consume the next input character:
                match self.read_next() {
                    Some(TAB | LINE_FEED | FORM_FEED | SPACE) => {}, // Ignore the character.
                    Some('/' | '>') | None => {
                        // Reconsume in the after attribute name state.
                        self.reconsume_in(TokenizerState::AfterAttributeNameState);
                    },
                    Some('=') => {
                        // This is an unexpected-equals-sign-before-attribute-name parse error.
                        // Start a new attribute in the current tag token.
                        self.get_current_tag().new_attribute();

                        // Set that attribute's name to the current input character, and its value to the empty string.
                        self.get_current_tag().add_to_attr_name('=');

                        // Switch to the attribute name state.
                        self.switch_to(TokenizerState::AttributeNameState);
                    },
                    _ => {
                        // Start a new attribute in the current tag token. Set that attribute name
                        // and value to the empty string.
                        self.get_current_tag().new_attribute();

                        // Reconsume in the attribute name state.
                        self.reconsume_in(TokenizerState::AttributeNameState);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#attribute-name-state
            TokenizerState::AttributeNameState => {
                // TODO: when leaving the AttributeNameState, we need to check
                // for duplicate attribute names.
                // https://html.spec.whatwg.org/multipage/parsing.html#attribute-name-state
                //
                // Consume the next input character:
                match self.read_next() {
                    Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' | '/' | '>') | None => {
                        // Reconsume in the after attribute name state.
                        self.reconsume_in(TokenizerState::AfterAttributeNameState);
                    },
                    Some('=') => {
                        // Switch to the before attribute value state.
                        self.switch_to(TokenizerState::BeforeAttributeValueState);
                    },
                    Some(c @ 'A'..='Z') => {
                        // Append the lowercase version of the current input character (add 0x0020
                        // to the character's code point) to the current attribute's name.
                        self.get_current_tag()
                            .add_to_attr_name(c.to_ascii_lowercase());
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        // Append a U+FFFD REPLACEMENT CHARACTER character to the current attribute's name.
                        self.get_current_tag().add_to_attr_name(UNICODE_REPLACEMENT);
                    },
                    Some(c @ ('"' | '\'' | '<')) => {
                        // This is an unexpected-character-in-attribute-name parse error.
                        // Treat it as per the "anything else" entry below.
                        self.get_current_tag().add_to_attr_name(c);
                    },
                    Some(c) => {
                        // Append the current input character to the current attribute's name.
                        self.get_current_tag().add_to_attr_name(c);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#after-attribute-name-state
            TokenizerState::AfterAttributeNameState => {
                // Consume the next input character:
                match self.read_next() {
                    Some(TAB | LINE_FEED | FORM_FEED | SPACE) => {}, // Ignore the character.
                    Some('/') => {
                        // Switch to the self-closing start tag state.
                        self.switch_to(TokenizerState::SelfClosingStartTagState);
                    },
                    Some('=') => {
                        // Switch to the before attribute value state.
                        self.switch_to(TokenizerState::BeforeAttributeValueState);
                    },
                    Some('>') => {
                        // Switch to the data state. Emit the current tag token.
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    },
                    Some(_) => {
                        // Start a new attribute in the current tag token.
                        // Set that attribute name and value to the empty string.
                        self.get_current_tag().new_attribute();

                        // Reconsume in the attribute name state.
                        self.reconsume_in(TokenizerState::AttributeNameState);
                    },
                    None => {
                        // This is an eof-in-tag parse error.
                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#before-attribute-value-state
            TokenizerState::BeforeAttributeValueState => {
                // Consume the next input character:
                match self.read_next() {
                    Some(TAB | LINE_FEED | FORM_FEED | SPACE) => {}, // Ignore the character.
                    Some('"') => {
                        // Switch to the attribute value (double-quoted) state.
                        self.switch_to(TokenizerState::AttributeValueDoublequotedState);
                    },
                    Some('\'') => {
                        // Switch to the attribute value (single-quoted) state.
                        self.switch_to(TokenizerState::AttributeValueSinglequotedState);
                    },
                    Some('>') => {
                        // This is a missing-attribute-value parse error.
                        // Switch to the data state.
                        self.switch_to(TokenizerState::DataState);

                        // Emit the current tag token.
                        self.emit_current_token();
                    },
                    _ => {
                        // Reconsume in the attribute value (unquoted) state.
                        self.reconsume_in(TokenizerState::AttributeValueUnquotedState);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(double-quoted)-state
            TokenizerState::AttributeValueDoublequotedState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('"') => {
                        // Switch to the after attribute value (quoted) state.
                        self.switch_to(TokenizerState::AfterAttributeValueQuotedState);
                    },
                    Some('&') => {
                        // Set the return state to the attribute value (double-quoted) state.
                        self.return_state = Some(TokenizerState::AttributeValueDoublequotedState);

                        // Switch to the character reference state.
                        self.switch_to(TokenizerState::CharacterReferenceState);
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        // Append a U+FFFD REPLACEMENT CHARACTER character to the current attribute's value.
                        self.get_current_tag()
                            .add_to_attr_value(UNICODE_REPLACEMENT);
                    },
                    Some(c) => {
                        // Append the current input character to the current attribute's value.
                        self.get_current_tag().add_to_attr_value(c);
                    },
                    None => {
                        // This is an eof-in-tag parse error.
                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(single-quoted)-state
            TokenizerState::AttributeValueSinglequotedState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('\'') => {
                        // Switch to the after attribute value (quoted) state.
                        self.switch_to(TokenizerState::AfterAttributeValueQuotedState);
                    },
                    Some('&') => {
                        // Set the return state to the attribute value (single-quoted) state.
                        self.return_state = Some(TokenizerState::AttributeValueSinglequotedState);

                        // Switch to the character reference state.
                        self.switch_to(TokenizerState::CharacterReferenceState);
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        // Append a U+FFFD REPLACEMENT CHARACTER character to the current attribute's value.
                        self.get_current_tag()
                            .add_to_attr_value(UNICODE_REPLACEMENT);
                    },
                    Some(c) => {
                        // Append the current input character to the current attribute's value.
                        self.get_current_tag().add_to_attr_value(c);
                    },
                    None => {
                        // This is an eof-in-tag parse error.
                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(unquoted)-state
            TokenizerState::AttributeValueUnquotedState => {
                // Consume the next input character:
                match self.read_next() {
                    Some(TAB | LINE_FEED | FORM_FEED | SPACE) => {
                        // Switch to the before attribute name state.
                        self.switch_to(TokenizerState::BeforeAttributeNameState);
                    },
                    Some('&') => {
                        // Set the return state to the attribute value (unquoted) state.
                        self.return_state = Some(TokenizerState::AttributeValueUnquotedState);

                        // Switch to the character reference state.
                        self.switch_to(TokenizerState::CharacterReferenceState);
                    },
                    Some('>') => {
                        // Switch to the data state. Emit the current tag token.
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        // Append a U+FFFD REPLACEMENT CHARACTER character to the current attribute's value.
                        self.get_current_tag()
                            .add_to_attr_value(UNICODE_REPLACEMENT);
                    },
                    Some(c @ ('"' | '\'' | '<' | '=' | '`')) => {
                        // This is an unexpected-character-in-unquoted-attribute-value parse error.
                        // Treat it as per the "anything else" entry below.
                        self.get_current_tag().add_to_attr_value(c);
                    },
                    Some(c) => {
                        // Append the current input character to the current attribute's value.
                        self.get_current_tag().add_to_attr_value(c);
                    },
                    None => {
                        // This is an eof-in-tag parse error. Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#after-attribute-value-(quoted)-state
            TokenizerState::AfterAttributeValueQuotedState => {
                // Consume the next input character:
                match self.read_next() {
                    Some(TAB | LINE_FEED | FORM_FEED | SPACE) => {
                        // Switch to the before attribute name state.
                        self.switch_to(TokenizerState::BeforeAttributeNameState);
                    },
                    Some('/') => {
                        // Switch to the self-closing start tag state.
                        self.switch_to(TokenizerState::SelfClosingStartTagState);
                    },
                    Some('>') => {
                        // Switch to the data state. Emit the current tag token.
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    },
                    Some(_) => {
                        // This is a missing-whitespace-between-attributes parse error.
                        // Reconsume in the before attribute name state.
                        self.reconsume_in(TokenizerState::BeforeAttributeNameState);
                    },
                    None => {
                        // This is an eof-in-tag parse error.
                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#self-closing-start-tag-state
            TokenizerState::SelfClosingStartTagState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('>') => {
                        // Set the self-closing flag of the current tag token.
                        self.get_current_tag().self_closing = true;

                        // Switch to the data state.
                        self.switch_to(TokenizerState::DataState);

                        // Emit the current tag token.
                        self.emit_current_token();
                    },
                    Some(_) => {
                        // This is an unexpected-solidus-in-tag parse error.
                        // Reconsume in the before attribute name state.
                        self.reconsume_in(TokenizerState::BeforeAttributeNameState);
                    },
                    None => {
                        // This is an eof-in-tag parse error.
                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#bogus-comment-state
            TokenizerState::BogusCommentState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('>') => {
                        // Switch to the data state.
                        self.switch_to(TokenizerState::DataState);

                        // Emit the current comment token.
                        self.emit_current_token();
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error. Append a U+FFFD
                        // REPLACEMENT CHARACTER character to the comment token's data.
                        self.get_current_comment().push(UNICODE_REPLACEMENT);
                    },
                    Some(c) => {
                        // Append the current input character to the comment token's data.
                        self.get_current_comment().push(c);
                    },
                    None => {
                        // Emit the comment.
                        self.emit_current_token();

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#markup-declaration-open-state
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
                    // This is an incorrectly-opened-comment parse error.
                    // Create a comment token whose data is the empty string.
                    self.current_token = Some(Token::Comment(String::default()));

                    // Switch to the bogus comment state (don't consume anything in the current state).
                    self.switch_to(TokenizerState::BogusCommentState);
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#comment-start-state
            TokenizerState::CommentStartState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('-') => {
                        // Switch to the comment start dash state.
                        self.switch_to(TokenizerState::CommentStartDashState);
                    },
                    Some('>') => {
                        // This is an abrupt-closing-of-empty-comment parse error.
                        // Switch to the data state.
                        self.switch_to(TokenizerState::DataState);

                        // Emit the current comment token.
                        self.emit_current_token();
                    },
                    _ => {
                        // Reconsume in the comment state.
                        self.reconsume_in(TokenizerState::CommentState);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#comment-start-dash-state
            TokenizerState::CommentStartDashState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('-') => {
                        // Switch to the comment end state.
                        self.switch_to(TokenizerState::CommentEndState);
                    },
                    Some('>') => {
                        // This is an abrupt-closing-of-empty-comment parse error.
                        // Switch to the data state.
                        self.switch_to(TokenizerState::DataState);

                        // Emit the current comment token.
                        self.emit_current_token();
                    },
                    Some(_) => {
                        // Append a U+002D HYPHEN-MINUS character (-) to the comment token's data.
                        self.get_current_comment().push('-');

                        // Reconsume in the comment state.
                        self.reconsume_in(TokenizerState::CommentState);
                    },
                    None => {
                        // This is an eof-in-comment parse error.
                        // Emit the current comment token.
                        self.emit_current_token();

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#comment-state
            TokenizerState::CommentState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('>') => {
                        // Append the current input character to the comment token's data.
                        self.get_current_comment().push('<');

                        // Switch to the comment less-than sign state.
                        self.switch_to(TokenizerState::CommentLessThanSignState);
                    },
                    Some('-') => {
                        // Switch to the comment end dash state.
                        self.switch_to(TokenizerState::CommentEndDashState);
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error. Append a U+FFFD
                        // REPLACEMENT CHARACTER character to the comment token's data.
                        self.get_current_comment().push(UNICODE_REPLACEMENT);
                    },
                    Some(c) => {
                        // Append the current input character to the comment token's data.
                        self.get_current_comment().push(c);
                    },
                    None => {
                        // This is an eof-in-comment parse error.
                        // Emit the current comment token.
                        self.emit_current_token();

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#comment-less-than-sign-state
            TokenizerState::CommentLessThanSignState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('!') => {
                        // Append the current input character to the comment token's data.
                        self.get_current_comment().push('!');

                        // Switch to the comment less-than sign bang state.
                        self.switch_to(TokenizerState::CommentLessThanSignBangState);
                    },
                    Some('<') => {
                        // Append the current input character to the comment token's data.
                        self.get_current_comment().push('<');
                    },
                    _ => {
                        // Reconsume in the comment state.
                        self.reconsume_in(TokenizerState::CommentState);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#comment-less-than-sign-bang-state
            TokenizerState::CommentLessThanSignBangState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('-') => {
                        // Switch to the comment less-than sign bang dash state.
                        self.switch_to(TokenizerState::CommentLessThanSignBangDashState);
                    },
                    _ => {
                        // Reconsume in the comment state.
                        self.reconsume_in(TokenizerState::CommentState);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#comment-less-than-sign-bang-dash-state
            TokenizerState::CommentLessThanSignBangDashState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('-') => {
                        // Switch to the comment less-than sign bang dash dash state.
                        self.switch_to(TokenizerState::CommentLessThanSignBangDashDashState);
                    },
                    _ => {
                        // Reconsume in the comment end dash state.
                        self.reconsume_in(TokenizerState::CommentEndDashState);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#comment-less-than-sign-bang-dash-dash-state
            TokenizerState::CommentLessThanSignBangDashDashState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('>') | None => {
                        // Reconsume in the comment end state.
                        self.reconsume_in(TokenizerState::CommentEndState);
                    },
                    Some(_) => {
                        // This is a nested-comment parse error.
                        // Reconsume in the comment end state.
                        self.reconsume_in(TokenizerState::CommentEndState);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#comment-end-dash-state
            TokenizerState::CommentEndDashState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('-') => {
                        // Switch to the comment end state.
                        self.switch_to(TokenizerState::CommentEndState);
                    },
                    Some(_) => {
                        // Append a U+002D HYPHEN-MINUS character (-) to the comment token's data.
                        self.get_current_comment().push('-');

                        // Reconsume in the comment state.
                        self.reconsume_in(TokenizerState::CommentState);
                    },
                    None => {
                        // This is an eof-in-comment parse error.
                        // Emit the current comment token.
                        self.emit_current_token();

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#comment-end-state
            TokenizerState::CommentEndState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('>') => {
                        // Switch to the data state. Emit the current comment token.
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    },
                    Some('!') => {
                        // Switch to the comment end bang state.
                        self.switch_to(TokenizerState::CommentEndBangState);
                    },
                    Some('-') => {
                        // Append a U+002D HYPHEN-MINUS character (-) to the comment token's data.
                        self.get_current_comment().push('-');
                    },
                    Some(_) => {
                        // Append two U+002D HYPHEN-MINUS characters (-) to the comment token's
                        // data.
                        self.get_current_comment().push_str("--");

                        // Reconsume in the comment state.
                        self.reconsume_in(TokenizerState::CommentState);
                    },
                    None => {
                        // This is an eof-in-comment parse error.
                        // Emit the current comment token.
                        self.emit_current_token();

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#comment-end-bang-state
            TokenizerState::CommentEndBangState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('-') => {
                        // Append two U+002D HYPHEN-MINUS characters (-) and a U+0021 EXCLAMATION
                        // MARK character (!) to the comment token's data.
                        self.get_current_comment().push_str("--!");

                        // Switch to the comment end dash state.
                        self.switch_to(TokenizerState::CommentEndDashState);
                    },
                    Some('>') => {
                        // This is an incorrectly-closed-comment parse error.
                        // Switch to the data state.
                        self.switch_to(TokenizerState::DataState);

                        // Emit the current comment token.
                        self.emit_current_token();
                    },
                    Some(_) => {
                        // Append two U+002D HYPHEN-MINUS characters (-) and a U+0021 EXCLAMATION
                        // MARK character (!) to the comment token's data.
                        self.get_current_comment().push_str("--!");

                        // Reconsume in the comment state.
                        self.reconsume_in(TokenizerState::CommentState);
                    },
                    None => {
                        // This is an eof-in-comment parse error.
                        // Emit the current comment token.
                        self.emit_current_token();

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#doctype-state
            TokenizerState::DOCTYPEState => {
                // Consume the next input character:
                match self.read_next() {
                    Some(TAB | LINE_FEED | FORM_FEED | SPACE) => {
                        // Switch to the before DOCTYPE name state.
                        self.switch_to(TokenizerState::BeforeDOCTYPENameState);
                    },
                    Some('>') => {
                        // Reconsume in the before DOCTYPE name state.
                        self.reconsume_in(TokenizerState::BeforeDOCTYPENameState);
                    },
                    Some(_) => {
                        // This is a missing-whitespace-before-doctype-name parse error.
                        // Reconsume in the before DOCTYPE name state.
                        self.reconsume_in(TokenizerState::BeforeDOCTYPENameState);
                    },
                    None => {
                        // This is an eof-in-doctype parse error.
                        // Create a new DOCTYPE token.
                        self.current_token = Some(Token::DOCTYPE(Doctype::default()));

                        // Set its force-quirks flag to on.
                        self.get_current_doctype().force_quirks = true;

                        // Emit the current token.
                        self.emit_current_token();

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#before-doctype-name-state
            TokenizerState::BeforeDOCTYPENameState => {
                // Note: this code potentially emits tokens *without* modifying self.current_token!

                // Consume the next input character:
                match self.read_next() {
                    Some(TAB | LINE_FEED | FORM_FEED | SPACE) => {}, // Ignore the character.
                    Some(c @ 'A'..='Z') => {
                        // Create a new DOCTYPE token.
                        // Set the token's name to the lowercase version of the current input character (add 0x0020 to the
                        // character's code point).
                        let doctype_token = Doctype {
                            name: Some(c.to_ascii_lowercase().to_string()),
                            ..Default::default()
                        };
                        self.current_token = Some(Token::DOCTYPE(doctype_token));

                        // Switch to the DOCTYPE name state.
                        self.switch_to(TokenizerState::DOCTYPENameState);
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.

                        // Create a new DOCTYPE token.
                        // Set the token's name to a U+FFFD REPLACEMENT CHARACTER character.
                        let doctype_token = Doctype {
                            name: Some(UNICODE_REPLACEMENT.to_string()),
                            ..Default::default()
                        };

                        self.current_token = Some(Token::DOCTYPE(doctype_token));

                        // Switch to the DOCTYPE name state.
                        self.switch_to(TokenizerState::DOCTYPENameState);
                    },
                    Some('>') => {
                        // This is a missing-doctype-name parse error.

                        // Create a new DOCTYPE token.
                        // Set its force-quirks flag to on.
                        let doctype_token = Doctype {
                            force_quirks: true,
                            ..Default::default()
                        };

                        // Switch to the data state.
                        self.switch_to(TokenizerState::DataState);

                        // Emit the current token.
                        self.emit(Token::DOCTYPE(doctype_token));
                    },
                    Some(c) => {
                        // Create a new DOCTYPE token.
                        // Set the token's name to the current input character.
                        let doctype_token = Doctype {
                            name: Some(c.to_string()),
                            ..Default::default()
                        };

                        self.current_token = Some(Token::DOCTYPE(doctype_token));

                        // Switch to the DOCTYPE name state.
                        self.switch_to(TokenizerState::DOCTYPENameState);
                    },
                    None => {
                        // This is an eof-in-doctype parse error.
                        // Create a new DOCTYPE token.
                        // Set its force-quirks flag to on.
                        let doctype_token = Doctype {
                            force_quirks: true,
                            ..Default::default()
                        };

                        // Emit the current token.
                        self.emit(Token::DOCTYPE(doctype_token));

                        // Emit an end-of-file token.
                        self.emit(Token::EOF)
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#doctype-name-state
            TokenizerState::DOCTYPENameState => {
                // Consume the next input character:
                match self.read_next() {
                    Some(TAB | LINE_FEED | FORM_FEED | SPACE) => {
                        // Switch to the after DOCTYPE name state.
                        self.switch_to(TokenizerState::AfterDOCTYPENameState);
                    },
                    Some('>') => {
                        // Switch to the data state. Emit the current DOCTYPE token.
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    },
                    Some(c @ 'A'..='Z') => {
                        // Append the lowercase version of the current input character (add 0x0020
                        // to the character's code point) to the current DOCTYPE token's name.
                        if let Some(name) = self.get_current_doctype().name.as_mut() {
                            name.push(c.to_ascii_lowercase())
                        }
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        // Append a U+FFFD REPLACEMENT CHARACTER character to the current DOCTYPE token's name.
                        if let Some(name) = self.get_current_doctype().name.as_mut() {
                            name.push(UNICODE_REPLACEMENT)
                        }
                    },
                    Some(c) => {
                        // Append the current input character to the current DOCTYPE token's name.
                        if let Some(name) = self.get_current_doctype().name.as_mut() {
                            name.push(c)
                        }
                    },
                    None => {
                        // This is an eof-in-doctype parse error.
                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.get_current_doctype().force_quirks = true;

                        // Emit the current DOCTYPE token.
                        self.emit_current_token();

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#after-doctype-name-state
            TokenizerState::AfterDOCTYPENameState => {
                // Consume the next input character:
                match self.read_next() {
                    Some(TAB | LINE_FEED | FORM_FEED | SPACE) => {}, // Ignore the character.
                    Some('>') => {
                        // Switch to the data state. Emit the current DOCTYPE token.
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    },
                    None => {
                        // This is an eof-in-doctype parse error.
                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.get_current_doctype().force_quirks = true;

                        // Emit the current DOCTYPE token.
                        self.emit_current_token();

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
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
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#after-doctype-public-keyword-state
            TokenizerState::AfterDOCTYPEPublicKeywordState => {
                // Consume the next input character:
                match self.read_next() {
                    Some(TAB | LINE_FEED | FORM_FEED | SPACE) => {
                        // Switch to the before DOCTYPE public identifier state.
                        self.switch_to(TokenizerState::BeforeDOCTYPEPublicIdentifierState);
                    },
                    Some('"') => {
                        // This is a missing-whitespace-after-doctype-public-keyword parse error.
                        // Set the current DOCTYPE token's public identifier to the empty string
                        // (not missing), then switch to the DOCTYPE public identifier
                        // (double-quoted) state.
                        self.get_current_doctype().public_ident = Some(String::new());
                        self.switch_to(TokenizerState::DOCTYPEPublicIdentifierDoublequotedState);
                    },
                    Some('\'') => {
                        // This is a missing-whitespace-after-doctype-public-keyword parse error.
                        // Set the current DOCTYPE token's public identifier to the empty string
                        // (not missing), then switch to the DOCTYPE public identifier
                        // (single-quoted) state.
                        self.get_current_doctype().public_ident = Some(String::new());
                        self.switch_to(TokenizerState::DOCTYPEPublicIdentifierSinglequotedState);
                    },
                    Some('>') => {
                        // This is a missing-doctype-public-identifier parse error. Set the current
                        // DOCTYPE token's force-quirks flag to on.
                        self.get_current_doctype().force_quirks = true;

                        // Switch to the data state.
                        self.switch_to(TokenizerState::DataState);

                        // Emit the current DOCTYPE token.
                        self.emit_current_token();
                    },
                    Some(_) => {
                        // This is a missing-quote-before-doctype-public-identifier parse error.
                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.get_current_doctype().force_quirks = true;

                        // Reconsume in the bogus DOCTYPE state.
                        self.reconsume_in(TokenizerState::BogusDOCTYPEState);
                    },
                    None => {
                        // This is an eof-in-doctype parse error. Set the current DOCTYPE token's
                        // force-quirks flag to on.
                        self.get_current_doctype().force_quirks = true;

                        // Emit the current DOCTYPE token.
                        self.emit_current_token();

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#before-doctype-public-identifier-state
            TokenizerState::BeforeDOCTYPEPublicIdentifierState => {
                // Consume the next input character:
                match self.read_next() {
                    Some(TAB | LINE_FEED | FORM_FEED | SPACE) => {}, // Ignore the character.
                    Some('"') => {
                        // Set the current DOCTYPE token's public identifier to the empty string
                        // (not missing), then switch to the DOCTYPE public identifier
                        // (double-quoted) state.
                        self.get_current_doctype().public_ident = Some(String::new());
                        self.switch_to(TokenizerState::DOCTYPEPublicIdentifierDoublequotedState);
                    },
                    Some('\'') => {
                        // Set the current DOCTYPE token's public identifier to the empty string
                        // (not missing), then switch to the DOCTYPE public identifier
                        // (single-quoted) state.
                        self.get_current_doctype().public_ident = Some(String::new());
                        self.switch_to(TokenizerState::DOCTYPEPublicIdentifierSinglequotedState);
                    },
                    Some('>') => {
                        // This is a missing-doctype-public-identifier parse error. Set the current
                        // DOCTYPE token's force-quirks flag to on.
                        self.get_current_doctype().force_quirks = true;

                        // Switch to the data state.
                        self.switch_to(TokenizerState::DataState);

                        // Emit the current DOCTYPE token.
                        self.emit_current_token();
                    },
                    Some(_) => {
                        // This is a missing-quote-before-doctype-public-identifier parse error.
                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.get_current_doctype().force_quirks = true;

                        // Reconsume in the bogus DOCTYPE state.
                        self.reconsume_in(TokenizerState::BogusDOCTYPEState);
                    },
                    None => {
                        // This is an eof-in-doctype parse error.
                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.get_current_doctype().force_quirks = true;

                        // Emit the current DOCTYPE token.
                        self.emit_current_token();

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#doctype-public-identifier-(double-quoted)-state
            TokenizerState::DOCTYPEPublicIdentifierDoublequotedState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('"') => {
                        // Switch to the after DOCTYPE public identifier state.
                        self.switch_to(TokenizerState::AfterDOCTYPEPublicIdentifierState);
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        // Append a U+FFFD REPLACEMENT CHARACTER character to the current DOCTYPE token's public
                        // identifier.
                        if let Some(ident) = self.get_current_doctype().public_ident.as_mut() {
                            ident.push(UNICODE_REPLACEMENT)
                        }
                    },
                    Some('>') => {
                        // This is an abrupt-doctype-public-identifier parse error.
                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.get_current_doctype().force_quirks = true;

                        // Switch to the data state.
                        self.switch_to(TokenizerState::DataState);

                        // Emit the current DOCTYPE token.
                        self.emit_current_token();
                    },
                    Some(c) => {
                        // Append the current input character to the current DOCTYPE token's
                        // public identifier.
                        if let Some(ident) = self.get_current_doctype().public_ident.as_mut() {
                            ident.push(c)
                        }
                    },
                    None => {
                        // This is an eof-in-doctype parse error.
                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.get_current_doctype().force_quirks = true;

                        // Emit the current DOCTYPE token.
                        self.emit_current_token();

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#doctype-public-identifier-(single-quoted)-state
            TokenizerState::DOCTYPEPublicIdentifierSinglequotedState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('\'') => {
                        // Switch to the after DOCTYPE public identifier state.
                        self.switch_to(TokenizerState::AfterDOCTYPEPublicIdentifierState);
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        // Append a U+FFFD REPLACEMENT CHARACTER character to the current DOCTYPE token's public
                        // identifier.
                        if let Some(ident) = self.get_current_doctype().public_ident.as_mut() {
                            ident.push(UNICODE_REPLACEMENT)
                        }
                    },
                    Some('>') => {
                        // This is an abrupt-doctype-public-identifier parse error.
                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.get_current_doctype().force_quirks = true;

                        // Switch to the data state.
                        self.switch_to(TokenizerState::DataState);

                        // Emit the current DOCTYPE token.
                        self.emit_current_token();
                    },
                    Some(c) => {
                        // Append the current input character to the current DOCTYPE token's public
                        // identifier.
                        if let Some(ident) = self.get_current_doctype().public_ident.as_mut() {
                            ident.push(c)
                        }
                    },
                    None => {
                        // This is an eof-in-doctype parse error.
                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.get_current_doctype().force_quirks = true;

                        // Emit the current DOCTYPE token.
                        self.emit_current_token();

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#after-doctype-public-identifier-state
            TokenizerState::AfterDOCTYPEPublicIdentifierState => {
                // Consume the next input character:
                match self.read_next() {
                    Some(TAB | LINE_FEED | FORM_FEED | SPACE) => {
                        // Switch to the between DOCTYPE public and system identifiers state.
                        self.switch_to(
                            TokenizerState::BetweenDOCTYPEPublicAndSystemIdentifiersState,
                        );
                    },
                    Some('>') => {
                        // Switch to the data state. Emit the current DOCTYPE token.
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    },
                    Some('"') => {
                        // This is a
                        // missing-whitespace-between-doctype-public-and-system-identifiers parse
                        // error.
                        // Set the current DOCTYPE token's system identifier to the empty
                        // string (not missing), then switch to the DOCTYPE system identifier
                        // (double-quoted) state.
                        self.get_current_doctype().system_ident = Some(String::new());
                        self.switch_to(TokenizerState::DOCTYPESystemIdentifierDoublequotedState);
                    },
                    Some('\'') => {
                        // This is a missing-whitespace-between-doctype-public-and-system-identifiers
                        // parse error.
                        // Set the current DOCTYPE token's system identifier to
                        // the empty string (not missing), then switch to the DOCTYPE system
                        // identifier (single-quoted) state.
                        self.get_current_doctype().system_ident = Some(String::new());
                        self.switch_to(TokenizerState::DOCTYPESystemIdentifierSinglequotedState);
                    },
                    Some(_) => {
                        // This is a missing-quote-before-doctype-system-identifier parse error.
                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.get_current_doctype().force_quirks = true;

                        // Reconsume in the bogus DOCTYPE state.
                        self.reconsume_in(TokenizerState::BogusDOCTYPEState);
                    },
                    None => {
                        // This is an eof-in-doctype parse error.
                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.get_current_doctype().force_quirks = true;

                        // Emit the current DOCTYPE token.
                        self.emit_current_token();

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#between-doctype-public-and-system-identifiers-state
            TokenizerState::BetweenDOCTYPEPublicAndSystemIdentifiersState => {
                // Consume the next input character:
                match self.read_next() {
                    Some(TAB | LINE_FEED | FORM_FEED | SPACE) => {}, // Ignore the character.
                    Some('>') => {
                        // Switch to the data state. Emit the current DOCTYPE token.
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    },
                    Some('"') => {
                        // Set the current DOCTYPE token's system identifier to the empty string
                        // (not missing), then switch to the DOCTYPE system identifier
                        // (double-quoted) state.
                        self.get_current_doctype().system_ident = Some(String::new());
                        self.switch_to(TokenizerState::DOCTYPESystemIdentifierDoublequotedState);
                    },
                    Some('\'') => {
                        // Set the current DOCTYPE token's system identifier to the empty string
                        // (not missing), then switch to the DOCTYPE system identifier
                        // (single-quoted) state.
                        self.get_current_doctype().system_ident = Some(String::new());
                        self.switch_to(TokenizerState::DOCTYPESystemIdentifierSinglequotedState);
                    },
                    Some(_) => {
                        // This is a missing-quote-before-doctype-system-identifier parse error.
                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.get_current_doctype().force_quirks = true;

                        // Reconsume in the bogus DOCTYPE state.
                        self.reconsume_in(TokenizerState::BogusDOCTYPEState);
                    },
                    None => {
                        // This is an eof-in-doctype parse error.
                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.get_current_doctype().force_quirks = true;

                        // Emit the current DOCTYPE token.
                        self.emit_current_token();

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#after-doctype-system-keyword-state
            TokenizerState::AfterDOCTYPESystemKeywordState => {
                // Consume the next input character:
                match self.read_next() {
                    Some(TAB | LINE_FEED | FORM_FEED | SPACE) => {
                        // Switch to the before DOCTYPE system identifier state.
                        self.switch_to(TokenizerState::BeforeDOCTYPESystemIdentifierState);
                    },
                    Some('"') => {
                        // This is a missing-whitespace-after-doctype-system-keyword parse error.
                        // Set the current DOCTYPE token's system identifier to the empty string
                        // (not missing), then switch to the DOCTYPE system identifier
                        // (double-quoted) state.
                        self.get_current_doctype().system_ident = Some(String::new());
                        self.switch_to(TokenizerState::DOCTYPESystemIdentifierDoublequotedState);
                    },
                    Some('\'') => {
                        // This is a missing-whitespace-after-doctype-system-keyword parse error.
                        // Set the current DOCTYPE token's system identifier to the empty string
                        // (not missing), then switch to the DOCTYPE system identifier
                        // (single-quoted) state.
                        self.get_current_doctype().system_ident = Some(String::new());
                        self.switch_to(TokenizerState::DOCTYPESystemIdentifierSinglequotedState);
                    },
                    Some('>') => {
                        // This is a missing-doctype-system-identifier parse error.
                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.get_current_doctype().force_quirks = true;

                        // Switch to the data state.
                        self.switch_to(TokenizerState::DataState);

                        // Emit the current DOCTYPE token.
                        self.emit_current_token();
                    },
                    Some(_) => {
                        // This is a missing-quote-before-doctype-system-identifier parse error.
                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.get_current_doctype().force_quirks = true;

                        // Reconsume in the bogus DOCTYPE state.
                        self.reconsume_in(TokenizerState::BogusDOCTYPEState);
                    },
                    None => {
                        // This is an eof-in-doctype parse error.
                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.get_current_doctype().force_quirks = true;

                        // Emit the current DOCTYPE token.
                        self.emit_current_token();

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#before-doctype-system-identifier-state
            TokenizerState::BeforeDOCTYPESystemIdentifierState => {
                // Consume the next input character:
                match self.read_next() {
                    Some(TAB | LINE_FEED | FORM_FEED | SPACE) => {}, //     Ignore the character.
                    Some('"') => {
                        // Set the current DOCTYPE token's system identifier to the empty string
                        // (not missing), then switch to the DOCTYPE system identifier
                        // (double-quoted) state.
                        self.get_current_doctype().system_ident = Some(String::new());
                        self.switch_to(TokenizerState::DOCTYPESystemIdentifierDoublequotedState);
                    },
                    Some('\'') => {
                        // Set the current DOCTYPE token's system identifier to the empty string
                        // (not missing), then switch to the DOCTYPE system identifier
                        // (single-quoted) state.
                        self.get_current_doctype().system_ident = Some(String::new());
                        self.switch_to(TokenizerState::DOCTYPESystemIdentifierSinglequotedState);
                    },
                    Some('>') => {
                        // This is a missing-doctype-system-identifier parse error.
                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.get_current_doctype().force_quirks = true;

                        // Switch to the data state.
                        self.switch_to(TokenizerState::DataState);

                        // Emit the current DOCTYPE token.
                        self.emit_current_token();
                    },
                    Some(_) => {
                        // This is a missing-quote-before-doctype-system-identifier parse error.
                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.get_current_doctype().force_quirks = true;

                        // Reconsume in the bogus DOCTYPE state.
                        self.reconsume_in(TokenizerState::BogusDOCTYPEState);
                    },
                    None => {
                        // This is an eof-in-doctype parse error. Set the current DOCTYPE token's
                        // force-quirks flag to on.
                        self.get_current_doctype().force_quirks = true;

                        // Emit the current DOCTYPE token.
                        self.emit_current_token();

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#doctype-system-identifier-(double-quoted)-state
            TokenizerState::DOCTYPESystemIdentifierDoublequotedState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('"') => {
                        // Switch to the after DOCTYPE system identifier state.
                        self.switch_to(TokenizerState::AfterDOCTYPESystemIdentifierState);
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        // Append a U+FFFD REPLACEMENT CHARACTER character to the current
                        // DOCTYPE token's system identifier.
                        if let Some(ident) = self.get_current_doctype().system_ident.as_mut() {
                            ident.push(UNICODE_REPLACEMENT)
                        }
                    },
                    Some('>') => {
                        // This is an abrupt-doctype-system-identifier parse error.
                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.get_current_doctype().force_quirks = true;

                        // Switch to the data state.
                        self.switch_to(TokenizerState::DataState);

                        // Emit the current DOCTYPE token.
                        self.emit_current_token();
                    },
                    Some(c) => {
                        // Append the current input character to the current DOCTYPE token's system
                        // identifier.
                        if let Some(ident) = self.get_current_doctype().system_ident.as_mut() {
                            ident.push(c)
                        }
                    },
                    None => {
                        // This is an eof-in-doctype parse error.
                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.get_current_doctype().force_quirks = true;

                        // Emit the current DOCTYPE token.
                        self.emit_current_token();

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#doctype-system-identifier-(single-quoted)-state
            TokenizerState::DOCTYPESystemIdentifierSinglequotedState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('\'') => {
                        // Switch to the after DOCTYPE system identifier state.
                        self.switch_to(TokenizerState::AfterDOCTYPESystemIdentifierState);
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        // Append a U+FFFD REPLACEMENT CHARACTER character to the current DOCTYPE token's system
                        // identifier.
                        if let Some(ident) = self.get_current_doctype().system_ident.as_mut() {
                            ident.push(UNICODE_REPLACEMENT)
                        }
                    },
                    Some('>') => {
                        // This is an abrupt-doctype-system-identifier parse error. Set the current
                        // DOCTYPE token's force-quirks flag to on.
                        self.get_current_doctype().force_quirks = true;

                        // Switch to the data state.
                        self.switch_to(TokenizerState::DataState);

                        // Emit the current DOCTYPE token.
                        self.emit_current_token();
                    },
                    Some(c) => {
                        // Append the current input character to the current DOCTYPE token's system
                        // identifier.
                        if let Some(ident) = self.get_current_doctype().system_ident.as_mut() {
                            ident.push(c)
                        }
                    },
                    None => {
                        // This is an eof-in-doctype parse error.
                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.get_current_doctype().force_quirks = true;

                        // Emit the current DOCTYPE token.
                        self.emit_current_token();

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#after-doctype-system-identifier-state
            TokenizerState::AfterDOCTYPESystemIdentifierState => {
                // Consume the next input character:
                match self.read_next() {
                    Some(TAB | LINE_FEED | FORM_FEED | SPACE) => {}, // Ignore the character.
                    Some('>') => {
                        // Switch to the data state.
                        self.switch_to(TokenizerState::DataState);

                        // Emit the current DOCTYPE token.
                        self.emit_current_token();
                    },
                    Some(_) => {
                        // This is an unexpected-character-after-doctype-system-identifier parse
                        // error.
                        // Reconsume in the bogus DOCTYPE state. (This does not set the
                        // current DOCTYPE token's force-quirks flag to on.)
                        self.reconsume_in(TokenizerState::BogusDOCTYPEState);
                    },
                    None => {
                        // This is an eof-in-doctype parse error. Set the current DOCTYPE
                        // token's force-quirks flag to on.
                        self.get_current_doctype().force_quirks = true;

                        // Emit the current DOCTYPE token.
                        self.emit_current_token();

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#bogus-doctype-state
            TokenizerState::BogusDOCTYPEState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('>') => {
                        // Switch to the data state. Emit the DOCTYPE token.
                        self.switch_to(TokenizerState::DataState);
                        self.emit_current_token();
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error. Ignore the character.
                    },
                    Some(_) => {}, // Ignore the character.
                    None => {
                        // Emit the DOCTYPE token.
                        self.emit_current_token();

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#cdata-section-state
            TokenizerState::CDATASectionState => {
                // Consume the next input character:
                match self.read_next() {
                    Some(']') => {
                        // Switch to the CDATA section bracket state.
                        self.switch_to(TokenizerState::CDATASectionBracketState);
                    },
                    Some(c) => {
                        // Emit the current input character as a character token.
                        self.emit(Token::Character(c));
                    },
                    None => {
                        // This is an eof-in-cdata parse error.
                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#cdata-section-bracket-state
            TokenizerState::CDATASectionBracketState => {
                // Consume the next input character:
                match self.read_next() {
                    Some(']') => {
                        // Switch to the CDATA section end state.
                        self.switch_to(TokenizerState::CDATASectionEndState);
                    },
                    _ => {
                        // Emit a U+005D RIGHT SQUARE BRACKET character token.
                        self.emit(Token::Character(']'));

                        // Reconsume in the CDATA section state.
                        self.reconsume_in(TokenizerState::CDATASectionState);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#cdata-section-end-state
            TokenizerState::CDATASectionEndState => {
                // Consume the next input character:
                match self.read_next() {
                    Some(']') => {
                        // Emit a U+005D RIGHT SQUARE BRACKET character token.
                        self.emit(Token::Character(']'));
                    },
                    Some('>') => {
                        // Switch to the data state.
                        self.switch_to(TokenizerState::DataState);
                    },
                    _ => {
                        // Emit two U+005D RIGHT SQUARE BRACKET character tokens.
                        self.emit(Token::Character(']'));
                        self.emit(Token::Character(']'));

                        // Reconsume in the CDATA section state.
                        self.reconsume_in(TokenizerState::CDATASectionState);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#character-reference-state
            TokenizerState::CharacterReferenceState => {
                // Set the temporary buffer to the empty string.
                // Append a U+0026 AMPERSAND (&) character to the temporary buffer.
                self.buffer = Some("&".to_string());

                // Consume the next input character:
                match self.read_next() {
                    Some('a'..='z' | 'A'..='Z' | '0'..='9') => {
                        // Reconsume in the named character reference state.
                        self.reconsume_in(TokenizerState::NamedCharacterReferenceState);
                    },
                    Some('#') => {
                        // Append the current input character to the temporary buffer.
                        self.add_to_buffer('#');

                        // Switch to the numeric character reference state.
                        self.switch_to(TokenizerState::NumericCharacterReferenceState);
                    },
                    _ => {
                        // Flush code points consumed as a character reference.

                        // we are supposed to flush the buffer as tokens - but we just set it to "&".
                        // Let's just emit a single '&' token i guess?
                        // Sorry to future me if this causes any bugs :^)
                        self.emit(Token::Character('&'));

                        // Reconsume in the return state.
                        self.reconsume_in(self.return_state.unwrap());
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#named-character-reference-state
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
                    },
                    None => {
                        // Flush code points consumed as a character reference.

                        // Switch to the ambiguous ampersand state.
                        self.switch_to(TokenizerState::AmbiguousAmpersandState);
                        todo!("flush code points consumed as character reference");
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#ambiguous-ampersand-state
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
                    },
                    Some(';') => {
                        // This is an unknown-named-character-reference parse error.
                        // Reconsume in the return state.
                        self.reconsume_in(self.return_state.unwrap());
                    },
                    _ => {
                        // Reconsume in the return state.
                        self.reconsume_in(self.return_state.unwrap());
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#numeric-character-reference-state
            TokenizerState::NumericCharacterReferenceState => {
                // Set the character reference code to zero (0).
                self.character_reference_code = 0;

                // Consume the next input character:
                match self.read_next() {
                    Some(c @ ('X' | 'x')) => {
                        // Append the current input character to the temporary buffer.
                        self.add_to_buffer(c);

                        // Switch to the hexadecimal character reference start state.
                        self.switch_to(TokenizerState::HexadecimalCharacterReferenceStartState);
                    },
                    _ => {
                        // Reconsume in the decimal character reference start state.
                        self.reconsume_in(TokenizerState::DecimalCharacterReferenceStartState);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#hexadecimal-character-reference-start-state
            TokenizerState::HexadecimalCharacterReferenceStartState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('0'..='9' | 'a'..='f' | 'A'..='F') => {
                        // Reconsume in the hexadecimal character reference state.
                        self.reconsume_in(TokenizerState::HexadecimalCharacterReferenceState);
                    },
                    _ => {
                        // This is an absence-of-digits-in-numeric-character-reference parse error.
                        // Flush code points consumed as a character reference.
                        // Reconsume in the return state.
                        self.reconsume_in(self.return_state.unwrap());
                        todo!("Flush code points consumed as a character reference.");
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#decimal-character-reference-start-state
            TokenizerState::DecimalCharacterReferenceStartState => {
                // Consume the next input character:
                match self.read_next() {
                    Some('0'..='9') => {
                        // Reconsume in the decimal character reference state.
                        self.reconsume_in(TokenizerState::DecimalCharacterReferenceState);
                    },
                    _ => {
                        // This is an absence-of-digits-in-numeric-character-reference parse
                        // error.
                        // Flush code points consumed as a character reference.
                        // Reconsume in the return state.
                        self.reconsume_in(self.return_state.unwrap());
                        todo!("Flush code points consumed as a character reference.");
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#hexadecimal-character-reference-state
            TokenizerState::HexadecimalCharacterReferenceState => {
                // Consume the next input character:
                match self.read_next() {
                    Some(c @ ('0'..='9' | 'a'..='f' | 'A'..='F')) => {
                        // Multiply the character reference code by 16.
                        self.character_reference_code *= 16;

                        // Add a numeric version of the current input character to the character reference code.
                        self.character_reference_code += c.to_digit(16).unwrap();
                    },
                    Some(';') => {
                        // Switch to the numeric character reference end state.
                        self.switch_to(TokenizerState::NumericCharacterReferenceEndState);
                    },
                    _ => {
                        // This is a missing-semicolon-after-character-reference parse error.
                        // Reconsume in the numeric character reference end state.
                        self.reconsume_in(TokenizerState::NumericCharacterReferenceEndState);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#decimal-character-reference-state
            TokenizerState::DecimalCharacterReferenceState => {
                // Consume the next input character:
                match self.read_next() {
                    Some(c @ '0'..='9') => {
                        // Multiply the character reference code by 10.
                        self.character_reference_code *= 10;

                        // Add a numeric version of
                        // the current input character (subtract 0x0030 from the character's code
                        // point) to the character reference code.
                        self.character_reference_code += c.to_digit(10).unwrap();
                    },
                    Some(';') => {
                        // Switch to the numeric character reference end state.
                        self.switch_to(TokenizerState::NumericCharacterReferenceEndState);
                    },
                    _ => {
                        // This is a missing-semicolon-after-character-reference parse error.
                        // Reconsume in the numeric character reference end state.
                        self.reconsume_in(TokenizerState::NumericCharacterReferenceEndState);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#numeric-character-reference-end-state
            TokenizerState::NumericCharacterReferenceEndState => {
                // Check the character reference code:
                match self.character_reference_code {
                    0x00 => {
                        // This is a null-character-reference parse error. Set the character
                        // reference code to 0xFFFD.
                        self.character_reference_code = 0xFFFD;
                    },
                    0x110000.. => {
                        // This is a character-reference-outside-unicode-range parse error. Set the
                        // character reference code to 0xFFFD.
                        self.character_reference_code = 0xFFFD;
                    },
                    0xD800..=0xDFFF => {
                        // This is a surrogate-character-reference parse error. Set the character
                        // reference code to 0xFFFD.
                        self.character_reference_code = 0xFFFD;
                    },
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
                    },
                    c @ (0x0D | 0xC0 | 0x007F..=0x009F) => {
                        if c != TAB as u32
                            || c != LINE_FEED as u32
                            || c != FORM_FEED as u32
                            || c != SPACE as u32
                        {
                            // control character reference parse error
                            match c {
                                0x80 => {
                                    self.character_reference_code = 0x20AC;
                                },
                                0x82 => {
                                    self.character_reference_code = 0x201A;
                                },
                                0x83 => {
                                    self.character_reference_code = 0x0192;
                                },
                                0x84 => {
                                    self.character_reference_code = 0x201E;
                                },
                                0x85 => {
                                    self.character_reference_code = 0x2026;
                                },
                                0x86 => {
                                    self.character_reference_code = 0x2020;
                                },
                                0x87 => {
                                    self.character_reference_code = 0x2021;
                                },
                                0x88 => {
                                    self.character_reference_code = 0x02C6;
                                },
                                0x89 => {
                                    self.character_reference_code = 0x2030;
                                },
                                0x8A => {
                                    self.character_reference_code = 0x0160;
                                },
                                0x8B => {
                                    self.character_reference_code = 0x2039;
                                },
                                0x8C => {
                                    self.character_reference_code = 0x0152;
                                },
                                0x8E => {
                                    self.character_reference_code = 0x017D;
                                },
                                0x91 => {
                                    self.character_reference_code = 0x2018;
                                },
                                0x92 => {
                                    self.character_reference_code = 0x2019;
                                },
                                0x93 => {
                                    self.character_reference_code = 0x201C;
                                },
                                0x94 => {
                                    self.character_reference_code = 0x201D;
                                },
                                0x95 => {
                                    self.character_reference_code = 0x2022;
                                },
                                0x96 => {
                                    self.character_reference_code = 0x2013;
                                },
                                0x97 => {
                                    self.character_reference_code = 0x2014;
                                },
                                0x98 => {
                                    self.character_reference_code = 0x02DC;
                                },
                                0x99 => {
                                    self.character_reference_code = 0x2122;
                                },
                                0x9A => {
                                    self.character_reference_code = 0x0161;
                                },
                                0x9B => {
                                    self.character_reference_code = 0x203A;
                                },
                                0x9C => {
                                    self.character_reference_code = 0x0153;
                                },
                                0x9E => {
                                    self.character_reference_code = 0x017E;
                                },
                                0x9F => {
                                    self.character_reference_code = 0x0178;
                                },
                                _ => {}, // no mapping
                            }
                        }
                    },
                    _ => {},
                }
                self.buffer = Some(
                    char::from_u32(self.character_reference_code)
                        .unwrap()
                        .to_string(),
                );
                self.switch_to(self.return_state.unwrap());
                todo!(); // flush, again
            },
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
