//! The [HTML Tokenizer](https://html.spec.whatwg.org/multipage/parsing.html#tokenization)
use sl_std::chars::ReversibleCharIterator;

use super::{
    lookup_character_reference,
    token::{CurrentToken, TagBuilder},
    HtmlParseError, ParseErrorHandler, TagData, Token,
};
use crate::infra;
use std::{collections::VecDeque, marker::PhantomData, mem};

// Characters that are hard to read
const UNICODE_REPLACEMENT: char = '\u{FFFD}';
const TAB: char = '\u{0009}';
const LINE_FEED: char = '\u{000A}';
const FORM_FEED: char = '\u{000C}';
const SPACE: char = '\u{0020}';

/// The different states of the [Tokenizer] state machine
#[derive(Debug, Clone, Copy)]
pub enum TokenizerState {
    /// <https://html.spec.whatwg.org/multipage/parsing.html#data-state>
    Data,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#rcdata-state>
    RCDATA,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#rawtext-state>
    RAWTEXT,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#script-data-state>
    ScriptData,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#plaintext-state>
    PLAINTEXT,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#tag-open-state>
    TagOpen,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#end-tag-open-state>
    EndTagOpen,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#tag-name-state>
    TagName,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#rcdata-less-than-sign-state>
    RCDATALessThanSign,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#rcdata-end-tag-open-state>
    RCDATAEndTagOpen,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#rcdata-end-tag-name-state>
    RCDATAEndTagName,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#rawtext-less-than-sign-state>
    RAWTEXTLessThanSign,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#rawtext-end-tag-open-state>
    RAWTEXTEndTagOpen,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#rawtext-end-tag-name-state>
    RAWTEXTEndTagName,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#script-data-less-than-sign-state>
    ScriptDataLessThanSign,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#script-data-end-tag-open-state>
    ScriptDataEndTagOpen,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#script-data-end-tag-name-state>
    ScriptDataEndTagName,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#script-data-escape-start-state>
    ScriptDataEscapeStart,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#script-data-escape-start-dash-state>
    ScriptDataEscapeStartDash,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#script-data-escaped-state>
    ScriptDataEscaped,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#script-data-escaped-dash-state>
    ScriptDataEscapedDash,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#script-data-escaped-dash-dash-state>
    ScriptDataEscapedDashDash,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#script-data-escaped-less-than-sign-state>
    ScriptDataEscapedLessThanSign,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#script-data-escaped-end-tag-open-state>
    ScriptDataEscapedEndTagOpen,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#script-data-escaped-end-tag-name-state>
    ScriptDataEscapedEndTagName,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#script-data-double-escape-start-state>
    ScriptDataDoubleEscapeStart,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#script-data-double-escaped-state>
    ScriptDataDoubleEscaped,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#script-data-double-escaped-dash-state>
    ScriptDataDoubleEscapedDash,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#script-data-double-escaped-dash-dash-state>
    ScriptDataDoubleEscapedDashDash,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#script-data-double-escaped-less-than-sign-state>
    ScriptDataDoubleEscapedLessThanSign,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#script-data-double-escape-end-state>
    ScriptDataDoubleEscapeEnd,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#before-attribute-name-state>
    BeforeAttributeName,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#attribute-name-state>
    AttributeName,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#after-attribute-name-state>
    AfterAttributeName,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#before-attribute-value-state>
    BeforeAttributeValue,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(double-quoted)-state>
    AttributeValueDoublequoted,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(single-quoted)-state>
    AttributeValueSinglequoted,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(unquoted)-state>
    AttributeValueUnquoted,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#after-attribute-value-(quoted)-state>
    AfterAttributeValueQuoted,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#self-closing-start-tag-state>
    SelfClosingStartTag,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#bogus-comment-state>
    BogusComment,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#markup-declaration-open-state>
    MarkupDeclarationOpen,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#comment-start-state>
    CommentStart,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#comment-start-dash-state>
    CommentStartDash,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#comment-state>
    Comment,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#comment-less-than-sign-state>
    CommentLessThanSign,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#comment-less-than-sign-bang-state>
    CommentLessThanSignBang,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#comment-less-than-sign-bang-dash-state>
    CommentLessThanSignBangDash,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#comment-less-than-sign-bang-dash-dash-state>
    CommentLessThanSignBangDashDash,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#comment-end-dash-state>
    CommentEndDash,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#comment-end-state>
    CommentEnd,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#comment-end-bang-state>
    CommentEndBang,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#doctype-state>
    DOCTYPE,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#before-doctype-name-state>
    BeforeDOCTYPEName,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#doctype-name-state>
    DOCTYPEName,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#after-doctype-name-state>
    AfterDOCTYPEName,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#after-doctype-public-keyword-state>
    AfterDOCTYPEPublicKeyword,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#before-doctype-public-identifier-state>
    BeforeDOCTYPEPublicIdentifier,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#doctype-public-identifier-(double-quoted)-state>
    DOCTYPEPublicIdentifierDoublequoted,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#doctype-public-identifier-(single-quoted)-state>
    DOCTYPEPublicIdentifierSinglequoted,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#after-doctype-public-identifier-state>
    AfterDOCTYPEPublicIdentifier,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#between-doctype-public-and-system-identifiers-state>
    BetweenDOCTYPEPublicAndSystemIdentifiers,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#after-doctype-system-keyword-state>
    AfterDOCTYPESystemKeyword,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#before-doctype-system-identifier-state>
    BeforeDOCTYPESystemIdentifier,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#doctype-system-identifier-(double-quoted)-state>
    DOCTYPESystemIdentifierDoublequoted,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#doctype-system-identifier-(single-quoted)-state>
    DOCTYPESystemIdentifierSinglequoted,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#after-doctype-system-identifier-state>
    AfterDOCTYPESystemIdentifier,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#bogus-doctype-state>
    BogusDOCTYPE,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#cdata-section-state>
    CDATASection,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#cdata-section-bracket-state>
    CDATASectionBracket,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#cdata-section-end-state>
    CDATASectionEnd,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#character-reference-state>
    CharacterReference,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#named-character-reference-state>
    NamedCharacterReference,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#ambiguous-ampersand-state>
    AmbiguousAmpersand,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#numeric-character-reference-state>
    NumericCharacterReference,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#hexadecimal-character-reference-start-state>
    HexadecimalCharacterReferenceStart,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#decimal-character-reference-start-state>
    DecimalCharacterReferenceStart,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#hexadecimal-character-reference-state>
    HexadecimalCharacterReference,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#decimal-character-reference-state>
    DecimalCharacterReference,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#numeric-character-reference-end-state>
    NumericCharacterReferenceEnd,
}

#[derive(Clone, Debug)]
pub struct Tokenizer<P: ParseErrorHandler> {
    /// The source code that is being tokenized
    source: ReversibleCharIterator<String>,

    /// The current state of the state machine
    state: TokenizerState,

    /// Whether or not we should continue tokenizing
    pub done: bool,

    /// The tokens produced by the [Tokenizer]
    token_buffer: VecDeque<Token>,

    /// Used by [TokenizerState::CharacterReference]
    return_state: Option<TokenizerState>,
    last_emitted_start_tag_name: Option<String>,

    /// The current tag being parsed, if any
    current_tag: TagBuilder,

    /// The current comment being parsed, if any
    current_comment: String,

    /// A general-purpose temporary buffer
    buffer: String,

    current_token: CurrentToken,
    character_reference_code: u32,
    phantom_data: PhantomData<P>,
}

impl<P: ParseErrorHandler> Tokenizer<P> {
    #[must_use]
    pub fn new(source: &str) -> Self {
        // Normalize newlines
        // https://infra.spec.whatwg.org/#normalize-newlines
        let source = infra::normalize_newlines(source);

        Self {
            source: ReversibleCharIterator::new(source),
            state: TokenizerState::Data,
            return_state: None,
            current_token: CurrentToken::default(),
            last_emitted_start_tag_name: None,
            character_reference_code: 0,

            current_tag: TagBuilder::default(),
            current_comment: String::new(),

            buffer: String::default(),
            done: false,
            token_buffer: VecDeque::new(),
            phantom_data: PhantomData,
        }
    }

    #[inline]
    fn parse_error(&mut self, variant: HtmlParseError) {
        P::handle(variant)
    }

    fn emit(&mut self, token: Token) {
        if let Token::Tag(TagData {
            opening: true,
            name,
            ..
        }) = &token
        {
            self.last_emitted_start_tag_name = Some(name.to_string());
        }
        self.token_buffer.push_back(token);
    }

    fn emit_current_token(&mut self) {
        let token = self.current_token.build();
        self.emit(token);
    }

    fn emit_current_tag_token(&mut self) {
        let tag = mem::take(&mut self.current_tag).finish();
        self.emit(tag);
    }

    fn emit_current_comment_token(&mut self) {
        let comment = Token::Comment(mem::take(&mut self.current_comment));
        self.emit(comment);
    }

    fn reconsume_in(&mut self, new_state: TokenizerState) {
        self.source.go_back();
        self.switch_to(new_state)
    }

    pub fn set_last_start_tag(&mut self, last_start_tag: Option<String>) {
        self.last_emitted_start_tag_name = last_start_tag;
    }

    /// Whether the current token is an [Token::EndTag] token whose name matches
    /// the name of the last [Token::StartTag] token that was emitted.
    #[must_use]
    fn is_appropriate_end_token(&self) -> bool {
        // Check if
        // * there was a start token emitted previously
        // * the token currently being emitted is an end token
        // * the name of the end token matches that of the start token
        self.last_emitted_start_tag_name
            .as_ref()
            .is_some_and(|last_emitted_start_tag_name| {
                !self.current_tag.is_opening
                    && &self.current_tag.name == last_emitted_start_tag_name
            })
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#flush-code-points-consumed-as-a-character-reference>
    fn flush_code_points_consumed_as_character_reference(&mut self) {
        let is_inside_attribute = self.is_inside_attribute();
        if is_inside_attribute {
            self.current_tag
                .current_attribute_value
                .push_str(&self.buffer);
        } else {
            mem::take(&mut self.buffer)
                .chars()
                .for_each(|c| self.emit(Token::Character(c)));
        }
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#charref-in-attribute>
    #[must_use]
    const fn is_inside_attribute(&self) -> bool {
        matches!(
            self.return_state,
            Some(
                TokenizerState::AttributeValueDoublequoted
                    | TokenizerState::AttributeValueSinglequoted
                    | TokenizerState::AttributeValueUnquoted
            )
        )
    }

    fn add_to_buffer(&mut self, c: char) {
        self.buffer.push(c);
    }

    /// Sets the current state to a specific state.
    /// All state transitions should call this method, which will
    /// ease debugging.
    pub fn switch_to(&mut self, state: TokenizerState) {
        self.state = state;
    }

    /// Reads the next character from the input stream
    #[must_use]
    fn read_next(&mut self) -> Option<char> {
        self.source.next()
    }

    pub fn step(&mut self) {
        match self.state {
            // https://html.spec.whatwg.org/multipage/parsing.html#data-state
            TokenizerState::Data => {
                // Consume the next input character:
                match self.read_next() {
                    Some('&') => {
                        // Set the return state to the data state.
                        self.return_state = Some(TokenizerState::Data);

                        // Switch to the character reference state.
                        self.switch_to(TokenizerState::CharacterReference);
                    },
                    Some('<') => {
                        // Switch to the tag open state.
                        self.switch_to(TokenizerState::TagOpen);
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        self.parse_error(HtmlParseError::UnexpectedNullCharacter);

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
            TokenizerState::RCDATA => {
                // Consume the next input character:
                match self.read_next() {
                    Some('&') => {
                        // Set the return state to the RCDATA state. Switch to the character
                        // reference state.
                        self.return_state = Some(TokenizerState::RCDATA);
                        self.switch_to(TokenizerState::CharacterReference);
                    },
                    Some('<') => {
                        // Switch to the RCDATA less-than sign state.
                        self.switch_to(TokenizerState::RCDATALessThanSign);
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        self.parse_error(HtmlParseError::UnexpectedNullCharacter);

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
            TokenizerState::RAWTEXT => {
                // Consume the next input character:
                match self.read_next() {
                    Some('<') => {
                        // Switch to the RAWTEXT less-than sign state.
                        self.switch_to(TokenizerState::RAWTEXTLessThanSign);
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        self.parse_error(HtmlParseError::UnexpectedNullCharacter);

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
            TokenizerState::ScriptData => {
                // Consume the next input character:
                match self.read_next() {
                    Some('<') => {
                        // Switch to the script data less-than sign state.
                        self.switch_to(TokenizerState::ScriptDataLessThanSign);
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        self.parse_error(HtmlParseError::UnexpectedNullCharacter);

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
            TokenizerState::PLAINTEXT => {
                // Consume the next input character:
                match self.read_next() {
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        self.parse_error(HtmlParseError::UnexpectedNullCharacter);

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
            TokenizerState::TagOpen => {
                // Consume the next input character:
                match self.read_next() {
                    Some('!') => {
                        // Switch to the markup declaration open state.
                        self.switch_to(TokenizerState::MarkupDeclarationOpen);
                    },
                    Some('/') => {
                        //     Switch to the end tag open state.
                        self.switch_to(TokenizerState::EndTagOpen);
                    },
                    Some('a'..='z' | 'A'..='Z') => {
                        // Create a new start tag token, set its tag name to the empty string.
                        self.current_tag = TagBuilder::opening();

                        // Reconsume in the tag name state.
                        self.reconsume_in(TokenizerState::TagName);
                    },
                    Some('?') => {
                        // This is an unexpected-question-mark-instead-of-tag-name parse error.
                        // Create a comment token whose data is the empty string.
                        self.current_comment.clear();

                        // Reconsume in the bogus comment state.
                        self.reconsume_in(TokenizerState::BogusComment);
                    },
                    Some(_) => {
                        // This is an invalid-first-character-of-tag-name parse error.
                        self.parse_error(HtmlParseError::InvalidFirstCharacterOfTagName);

                        // Emit a U+003C LESS-THAN SIGN character token.
                        self.emit(Token::Character('<'));

                        // Reconsume in the data state.
                        self.reconsume_in(TokenizerState::Data);
                    },
                    None => {
                        // This is an eof-before-tag-name parse error.
                        self.parse_error(HtmlParseError::EOFBeforeTagName);

                        // Emit a U+003C LESS-THAN SIGN
                        // character token and an end-of-file token.
                        self.emit(Token::Character('<'));
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#end-tag-open-state
            TokenizerState::EndTagOpen => {
                // Consume the next input character:
                match self.read_next() {
                    Some('a'..='z' | 'A'..='Z') => {
                        // Create a new end tag token, set its tag name to the empty string.
                        self.current_tag = TagBuilder::closing();

                        // Reconsume in the tag name state.
                        self.reconsume_in(TokenizerState::TagName);
                    },
                    Some('>') => {
                        // This is a missing-end-tag-name parse error.
                        self.parse_error(HtmlParseError::MissingEndTagName);

                        // Switch to the data state.
                        self.switch_to(TokenizerState::Data);
                    },
                    Some(_) => {
                        // This is an invalid-first-character-of-tag-name parse error.
                        self.parse_error(HtmlParseError::InvalidFirstCharacterOfTagName);

                        // Create a comment token whose data is the empty string.
                        self.current_comment.clear();

                        // Reconsume in the bogus comment state.
                        self.reconsume_in(TokenizerState::BogusComment);
                    },
                    None => {
                        // This is an eof-before-tag-name parse error.
                        self.parse_error(HtmlParseError::EOFBeforeTagName);

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
            TokenizerState::TagName => {
                // Consume the next input character:
                match self.read_next() {
                    Some(TAB | LINE_FEED | FORM_FEED | SPACE) => {
                        // Switch to the before attribute name state.
                        self.switch_to(TokenizerState::BeforeAttributeName);
                    },
                    Some('/') => {
                        // Switch to the self-closing start tag state.
                        self.switch_to(TokenizerState::SelfClosingStartTag);
                    },
                    Some('>') => {
                        // Switch to the data state. Emit the current tag token.
                        self.switch_to(TokenizerState::Data);
                        self.emit_current_tag_token();
                    },
                    Some(c @ 'A'..='Z') => {
                        // Append the lowercase version of the current input character (add
                        // 0x0020 to the character's code point) to the current tag token's tag
                        // name.
                        self.current_tag.name.push(c.to_ascii_lowercase());
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        self.parse_error(HtmlParseError::UnexpectedNullCharacter);

                        // Append a U+FFFD REPLACEMENT CHARACTER character to the current tag token's tag name.
                        self.current_tag.name.push(UNICODE_REPLACEMENT);
                    },
                    Some(c) => {
                        // Append the current input character to the current tag token's tag name.
                        self.current_tag.name.push(c);
                    },
                    None => {
                        // This is an eof-in-tag parse error.
                        self.parse_error(HtmlParseError::EOFInTag);

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#rcdata-less-than-sign-state
            TokenizerState::RCDATALessThanSign => {
                // Consume the next input character:
                match self.read_next() {
                    Some('/') => {
                        // Set the temporary buffer to the empty string.
                        self.buffer.clear();

                        // Switch to the RCDATA end tag open state.
                        self.switch_to(TokenizerState::RCDATAEndTagOpen);
                    },
                    _ => {
                        // Emit a U+003C LESS-THAN SIGN character token.
                        self.emit(Token::Character('<'));

                        // Reconsume in the RCDATA state.
                        self.reconsume_in(TokenizerState::RCDATA);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#rcdata-end-tag-open-state
            TokenizerState::RCDATAEndTagOpen => {
                // Consume the next input character:
                match self.read_next() {
                    Some('a'..='z' | 'A'..='Z') => {
                        // Create a new end tag token, set its tag name to the empty string.
                        self.current_tag = TagBuilder::closing();

                        // Reconsume in the RCDATA end tag name state.
                        self.reconsume_in(TokenizerState::RCDATAEndTagName);
                    },
                    _ => {
                        // Emit a U+003C LESS-THAN SIGN character token and a U+002F SOLIDUS
                        // character token.
                        self.emit(Token::Character('<'));
                        self.emit(Token::Character('/'));

                        // Reconsume in the RCDATA state.
                        self.reconsume_in(TokenizerState::RCDATA);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#rcdata-end-tag-name-state
            TokenizerState::RCDATAEndTagName => {
                // Consume the next input character:
                match (self.read_next(), self.is_appropriate_end_token()) {
                    (Some(TAB | LINE_FEED | FORM_FEED | SPACE), true) => {
                        // Switch to the before attribute name state.
                        self.switch_to(TokenizerState::BeforeAttributeName);
                    },
                    (Some('/'), true) => {
                        // Switch to the self-closing start tag state.
                        self.switch_to(TokenizerState::SelfClosingStartTag);
                    },
                    (Some('>'), true) => {
                        // Switch to the data state and emit the current tag token
                        self.switch_to(TokenizerState::Data);
                        self.emit_current_tag_token();
                    },
                    (Some(c @ 'A'..='Z'), _) => {
                        // Append the lowercase version of the current input character (add 0x0020
                        // to the character's code point) to the current tag token's tag name.
                        // Append the current input character to the temporary buffer.
                        self.add_to_buffer(c);
                        self.current_tag.name.push(c.to_ascii_lowercase());
                    },
                    (Some(c @ 'a'..='z'), _) => {
                        // Append the current input character to the current tag token's tag name.
                        self.current_tag.name.push(c);

                        // Append the current input character to the temporary buffer.
                        self.add_to_buffer(c);
                    },
                    _ => {
                        // Emit a U+003C LESS-THAN SIGN character token, a U+002F SOLIDUS character
                        // token, and a character token for each of the characters in the temporary
                        // buffer (in the order they were added to the buffer).
                        self.emit(Token::Character('<'));
                        self.emit(Token::Character('/'));
                        for c in mem::take(&mut self.buffer).chars() {
                            self.emit(Token::Character(c));
                        }

                        // Reconsume in the RCDATA state.
                        self.reconsume_in(TokenizerState::RCDATA);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#rawtext-less-than-sign-state
            TokenizerState::RAWTEXTLessThanSign => {
                // Consume the next input character:
                match self.read_next() {
                    Some('/') => {
                        // Set the temporary buffer to the empty string.
                        // Switch to the RAWTEXT end tag open state.
                        self.buffer.clear();
                        self.switch_to(TokenizerState::RAWTEXTEndTagOpen);
                    },
                    _ => {
                        // Emit a U+003C LESS-THAN SIGN character token.
                        self.emit(Token::Character('<'));

                        // Reconsume in the RAWTEXT state.
                        self.reconsume_in(TokenizerState::RAWTEXT);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#rawtext-end-tag-open-state
            TokenizerState::RAWTEXTEndTagOpen => {
                // Consume the next input character:
                match self.read_next() {
                    Some('a'..='z' | 'A'..='Z') => {
                        // Create a new end tag token, set its tag name to the empty string.
                        self.current_tag = TagBuilder::closing();

                        // Reconsume in the RAWTEXT end tag name state.
                        self.reconsume_in(TokenizerState::RAWTEXTEndTagName);
                    },
                    _ => {
                        // Emit a U+003C LESS-THAN SIGN character token and a U+002F SOLIDUS
                        // character token.
                        self.emit(Token::Character('<'));
                        self.emit(Token::Character('/'));

                        // Reconsume in the RAWTEXT state.
                        self.reconsume_in(TokenizerState::RAWTEXT);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#rawtext-end-tag-name-state
            TokenizerState::RAWTEXTEndTagName => {
                // Consume the next input character:
                match (self.read_next(), self.is_appropriate_end_token()) {
                    (Some(TAB | LINE_FEED | FORM_FEED | SPACE), true) => {
                        // Switch to the before attribute name state
                        self.switch_to(TokenizerState::BeforeAttributeName);
                    },
                    (Some('/'), true) => {
                        // Switch to the self-closing start tag state
                        self.switch_to(TokenizerState::SelfClosingStartTag);
                    },
                    (Some('>'), true) => {
                        // Switch to the data state and emit the current tag token
                        self.switch_to(TokenizerState::Data);
                        self.emit_current_tag_token();
                    },
                    (Some(c @ 'A'..='Z'), _) => {
                        // Append the lowercase version of the current input character (add 0x0020
                        // to the character's code point) to the current tag token's tag name.
                        self.current_tag.name.push(c.to_ascii_lowercase());

                        // Append the current input character to the temporary buffer.
                        self.add_to_buffer(c);
                    },
                    (Some(c @ 'a'..='z'), _) => {
                        // Append the current input character to the current tag token's tag name.
                        self.current_tag.name.push(c);

                        // Append the current input character to the temporary buffer.
                        self.add_to_buffer(c);
                    },
                    _ => {
                        // Emit a U+003C LESS-THAN SIGN character token, a U+002F SOLIDUS character
                        // token, and a character token for each of the characters in the temporary
                        // buffer (in the order they were added to the buffer).
                        self.emit(Token::Character('<'));
                        self.emit(Token::Character('/'));
                        for c in mem::take(&mut self.buffer).chars() {
                            self.emit(Token::Character(c));
                        }

                        // Reconsume in the RAWTEXT state.
                        self.reconsume_in(TokenizerState::RAWTEXT);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-less-than-sign-state
            TokenizerState::ScriptDataLessThanSign => {
                // Consume the next input character:
                match self.read_next() {
                    Some('/') => {
                        // Set the temporary buffer to the empty string.
                        self.buffer.clear();

                        // Switch to the script data end tag open state.
                        self.switch_to(TokenizerState::ScriptDataEndTagOpen);
                    },
                    Some('!') => {
                        // Switch to the script data escape start state.
                        self.switch_to(TokenizerState::ScriptDataEscapeStart);

                        // Emit a U+003C LESS-THAN SIGN character token and a
                        // U+0021 EXCLAMATION MARK character token.
                        self.emit(Token::Character('<'));
                        self.emit(Token::Character('!'));
                    },
                    _ => {
                        // Emit a U+003C LESS-THAN SIGN character token.
                        self.emit(Token::Character('<'));

                        // Reconsume in the script data state.
                        self.reconsume_in(TokenizerState::ScriptData);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-end-tag-open-state
            TokenizerState::ScriptDataEndTagOpen => {
                // Consume the next input character:
                match self.read_next() {
                    Some('a'..='z' | 'A'..='Z') => {
                        // Create a new end tag token, set its tag name to the empty string.
                        self.current_tag = TagBuilder::closing();

                        // Reconsume in the script data end tag name state.
                        self.reconsume_in(TokenizerState::ScriptDataEndTagName);
                    },
                    _ => {
                        // Emit a U+003C LESS-THAN SIGN character token and a U+002F SOLIDUS
                        // character token.
                        self.emit(Token::Character('<'));
                        self.emit(Token::Character('/'));

                        // Reconsume in the script data state.
                        self.reconsume_in(TokenizerState::ScriptData);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-end-tag-name-state
            TokenizerState::ScriptDataEndTagName => {
                // Consume the next input character:
                match (self.read_next(), self.is_appropriate_end_token()) {
                    (Some(TAB | LINE_FEED | FORM_FEED | SPACE), true) => {
                        // Switch to the before attribute name state
                        self.switch_to(TokenizerState::BeforeAttributeName);
                    },
                    (Some('/'), true) => {
                        // Switch to the self-closing start tag state
                        self.switch_to(TokenizerState::SelfClosingStartTag);
                    },
                    (Some('>'), true) => {
                        // Switch to the data state and emit the current tag token
                        self.switch_to(TokenizerState::Data);
                        self.emit_current_tag_token();
                    },
                    (Some(c @ 'A'..='Z'), _) => {
                        // Append the lowercase version of the current input character (add
                        // 0x0020 to the character's code point) to the current tag token's tag
                        // name.
                        self.current_tag.name.push(c.to_ascii_lowercase());

                        // Append the current input character to the temporary buffer.
                        self.add_to_buffer(c);
                    },
                    (Some(c @ 'a'..='z'), _) => {
                        // Append the current input character to the current tag token's tag name.
                        self.current_tag.name.push(c);

                        // Append the current input character to the temporary buffer.
                        self.add_to_buffer(c);
                    },
                    _ => {
                        // Emit a U+003C LESS-THAN SIGN character token, a U+002F SOLIDUS character
                        // token, and a character token for each of the characters in the temporary
                        // buffer (in the order they were added to the buffer).
                        self.emit(Token::Character('<'));
                        self.emit(Token::Character('/'));
                        for c in mem::take(&mut self.buffer).chars() {
                            self.emit(Token::Character(c));
                        }

                        // Reconsume in the script data state.
                        self.reconsume_in(TokenizerState::ScriptData);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escape-start-state
            TokenizerState::ScriptDataEscapeStart => {
                // Consume the next input character:
                match self.read_next() {
                    Some('-') => {
                        // Switch to the script data escape start dash state.
                        self.switch_to(TokenizerState::ScriptDataEscapeStartDash);

                        // Emit a U+002D HYPHEN-MINUS character token.
                        self.emit(Token::Character('-'));
                    },
                    _ => {
                        // Reconsume in the script data state.
                        self.reconsume_in(TokenizerState::ScriptData);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escape-start-dash-state
            TokenizerState::ScriptDataEscapeStartDash => {
                // Consume the next input character:
                match self.read_next() {
                    Some('-') => {
                        // Switch to the script data escaped dash dash state.
                        self.switch_to(TokenizerState::ScriptDataEscapedDashDash);

                        // Emit a U+002D HYPHEN-MINUS character token.
                        self.emit(Token::Character('-'));
                    },
                    _ => {
                        // Reconsume in the script data state.
                        self.reconsume_in(TokenizerState::ScriptData);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escaped-state
            TokenizerState::ScriptDataEscaped => {
                // Consume the next input character:
                match self.read_next() {
                    Some('-') => {
                        // Switch to the script data escaped dash state.
                        self.switch_to(TokenizerState::ScriptDataEscapedDash);

                        // Emit a U+002D HYPHEN-MINUS character token.
                        self.emit(Token::Character('-'));
                    },
                    Some('<') => {
                        // Switch to the script data escaped less-than sign state.
                        self.switch_to(TokenizerState::ScriptDataEscapedLessThanSign);
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        self.parse_error(HtmlParseError::UnexpectedNullCharacter);

                        // Emit a U+FFFD REPLACEMENT CHARACTER character token.
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    },
                    Some(c) => {
                        // Emit the current input character as a character token.
                        self.emit(Token::Character(c));
                    },
                    None => {
                        // This is an eof-in-script-html-comment-like-text parse error.
                        self.parse_error(HtmlParseError::EOFInScriptHtmlCommentLikeText);

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escaped-dash-state
            TokenizerState::ScriptDataEscapedDash => {
                // Consume the next input character:
                match self.read_next() {
                    Some('-') => {
                        // Switch to the script data escaped dash dash state.
                        self.switch_to(TokenizerState::ScriptDataEscapedDashDash);

                        // Emit a U+002D HYPHEN-MINUS character token.
                        self.emit(Token::Character('-'));
                    },
                    Some('<') => {
                        // Switch to the script data escaped less-than sign state.
                        self.switch_to(TokenizerState::ScriptDataEscapedLessThanSign);
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        self.parse_error(HtmlParseError::UnexpectedNullCharacter);

                        // Emit a U+FFFD REPLACEMENT CHARACTER character token.
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    },
                    Some(c) => {
                        // Switch to the script data escaped state.
                        self.switch_to(TokenizerState::ScriptDataEscaped);

                        // Emit the current input character as a character token.
                        self.emit(Token::Character(c));
                    },
                    None => {
                        // This is an eof-in-script-html-comment-like-text parse error.
                        self.parse_error(HtmlParseError::EOFInScriptHtmlCommentLikeText);

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escaped-dash-dash-state
            TokenizerState::ScriptDataEscapedDashDash => {
                // Consume the next input character:
                match self.read_next() {
                    Some('-') => {
                        // Emit a U+002D HYPHEN-MINUS character token.
                        self.emit(Token::Character('-'));
                    },
                    Some('<') => {
                        // Switch to the script data escaped less-than sign state.
                        self.switch_to(TokenizerState::ScriptDataEscapedLessThanSign);
                    },
                    Some('>') => {
                        // Switch to the script data state.
                        self.switch_to(TokenizerState::ScriptData);

                        // Emit a U+003E GREATER-THAN SIGN character token.
                        self.emit(Token::Character('>'));
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        self.parse_error(HtmlParseError::UnexpectedNullCharacter);

                        // Emit a U+FFFD REPLACEMENT CHARACTER character token.
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    },
                    Some(c) => {
                        // Switch to the script data escaped state.
                        self.switch_to(TokenizerState::ScriptDataEscaped);

                        // Emit the current input character as a character token.
                        self.emit(Token::Character(c));
                    },
                    None => {
                        // This is an eof-in-script-html-comment-like-text parse error.
                        self.parse_error(HtmlParseError::EOFInScriptHtmlCommentLikeText);

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escaped-less-than-sign-state
            TokenizerState::ScriptDataEscapedLessThanSign => {
                // Consume the next input character:
                match self.read_next() {
                    Some('/') => {
                        // Set the temporary buffer to the empty string.
                        self.buffer.clear();

                        // Switch to the script data escaped end tag open state.
                        self.switch_to(TokenizerState::ScriptDataEscapedEndTagOpen);
                    },
                    Some('a'..='z' | 'A'..='Z') => {
                        // Set the temporary buffer to the empty string.
                        self.buffer.clear();

                        // Emit a U+003C LESS-THAN SIGN character token.
                        self.emit(Token::Character('<'));

                        // Reconsume in the script data double escape start state.
                        self.reconsume_in(TokenizerState::ScriptDataDoubleEscapeStart);
                    },
                    _ => {
                        // Emit a U+003C LESS-THAN SIGN character token.
                        self.emit(Token::Character('<'));

                        // Reconsume in the script data escaped state.
                        self.reconsume_in(TokenizerState::ScriptDataEscaped);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escaped-end-tag-open-state
            TokenizerState::ScriptDataEscapedEndTagOpen => {
                // Consume the next input character:
                match self.read_next() {
                    Some('a'..='z' | 'A'..='Z') => {
                        // Create a new end tag token, set its tag name to the empty string.
                        self.current_tag = TagBuilder::closing();

                        // Reconsume in the script data escaped end tag name state.
                        self.reconsume_in(TokenizerState::ScriptDataEscapedEndTagName);
                    },
                    _ => {
                        // Emit a U+003C LESS-THAN SIGN character token and a U+002F SOLIDUS
                        // character token.
                        self.emit(Token::Character('<'));
                        self.emit(Token::Character('/'));

                        // Reconsume in the script data escaped state.
                        self.reconsume_in(TokenizerState::ScriptDataEscaped);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escaped-end-tag-name-state
            TokenizerState::ScriptDataEscapedEndTagName => {
                // Consume the next input character:
                match self.read_next() {
                    Some(c @ 'A'..='Z') => {
                        // Append the lowercase version of the current input character (add 0x0020
                        // to the character's code point) to the current tag token's tag name.
                        self.current_tag.name.push(c.to_ascii_lowercase());

                        // Append the current input character to the temporary buffer.
                        self.add_to_buffer(c);
                    },
                    Some(c @ 'a'..='z') => {
                        // Append the current input character to the current tag token's tag name.
                        self.current_tag.name.push(c);

                        // Append the current input character to the temporary buffer.
                        self.add_to_buffer(c);
                    },
                    other => {
                        match (other, self.is_appropriate_end_token()) {
                            (Some(TAB | LINE_FEED | FORM_FEED | SPACE), true) => {
                                // Switch to the before attribute name state.
                                self.switch_to(TokenizerState::BeforeAttributeName);
                            },
                            (Some('/'), true) => {
                                // Switch to the self-closing start tag state.
                                self.switch_to(TokenizerState::SelfClosingStartTag);
                            },
                            (Some('>'), true) => {
                                // Switch to the data state and emit the current tag token.
                                self.switch_to(TokenizerState::Data);
                                self.emit_current_tag_token();
                            },
                            _ => {
                                // Emit a U+003C LESS-THAN SIGN character token, a U+002F SOLIDUS character
                                // token, and a character token for each of the characters in the temporary
                                // buffer (in the order they were added to the buffer).
                                self.emit(Token::Character('<'));
                                self.emit(Token::Character('/'));
                                for c in mem::take(&mut self.buffer).chars() {
                                    self.emit(Token::Character(c));
                                }

                                // Reconsume in the script data escaped state.
                                self.reconsume_in(TokenizerState::ScriptDataEscaped);
                            },
                        }
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-double-escape-start-state
            TokenizerState::ScriptDataDoubleEscapeStart => {
                // Consume the next input character:
                match self.read_next() {
                    Some(c @ ('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' | '/' | '>')) => {
                        // If the temporary buffer is the string "script",
                        if self.buffer == "script" {
                            // then switch to the script data double escaped state.
                            self.switch_to(TokenizerState::ScriptDataDoubleEscaped);
                        } else {
                            // Otherwise, switch to the script data escaped state.
                            self.switch_to(TokenizerState::ScriptDataEscaped);
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
                        self.reconsume_in(TokenizerState::ScriptDataEscaped);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-double-escaped-state
            TokenizerState::ScriptDataDoubleEscaped => {
                // Consume the next input character:
                match self.read_next() {
                    Some('-') => {
                        // Switch to the script data double escaped dash state.
                        self.switch_to(TokenizerState::ScriptDataDoubleEscapedDash);

                        // Emit a U+002D HYPHEN-MINUS character token.
                        self.emit(Token::Character('-'));
                    },
                    Some('<') => {
                        // Switch to the script data double escaped less-than sign state.
                        self.switch_to(TokenizerState::ScriptDataDoubleEscapedLessThanSign);

                        // Emit a U+003C LESS-THAN SIGN character token.
                        self.emit(Token::Character('<'));
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        self.parse_error(HtmlParseError::UnexpectedNullCharacter);

                        // Emit a U+FFFD REPLACEMENT CHARACTER character token.
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    },
                    Some(c) => {
                        // Emit the current input character as a character token.
                        self.emit(Token::Character(c));
                    },
                    None => {
                        // This is an eof-in-script-html-comment-like-text parse error.
                        self.parse_error(HtmlParseError::EOFInScriptHtmlCommentLikeText);

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-double-escaped-dash-state
            TokenizerState::ScriptDataDoubleEscapedDash => {
                // Consume the next input character:
                match self.read_next() {
                    Some('-') => {
                        // Switch to the script data double escaped dash dash state.
                        self.switch_to(TokenizerState::ScriptDataDoubleEscapedDashDash);

                        // Emit a U+002D HYPHEN-MINUS character token.
                        self.emit(Token::Character('-'));
                    },
                    Some('<') => {
                        // Switch to the script data double escaped less-than sign state.
                        self.switch_to(TokenizerState::ScriptDataDoubleEscapedLessThanSign);

                        // Emit a U+003C LESS-THAN SIGN character token.
                        self.emit(Token::Character('<'));
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        self.parse_error(HtmlParseError::UnexpectedNullCharacter);

                        // Switch to the script data double escaped state.
                        self.switch_to(TokenizerState::ScriptDataDoubleEscaped);

                        // Emit a U+FFFD REPLACEMENT CHARACTER character token.
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    },
                    Some(c) => {
                        // Switch to the script data double escaped state.
                        self.switch_to(TokenizerState::ScriptDataDoubleEscaped);

                        // Emit the current input character as a character token.
                        self.emit(Token::Character(c));
                    },
                    None => {
                        // This is an eof-in-script-html-comment-like-text parse error.
                        self.parse_error(HtmlParseError::EOFInScriptHtmlCommentLikeText);

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-double-escaped-dash-dash-state
            TokenizerState::ScriptDataDoubleEscapedDashDash => {
                // Consume the next input character:
                match self.read_next() {
                    Some('-') => {
                        // Emit a U+002D HYPHEN-MINUS character token.
                        self.emit(Token::Character('-'));
                    },
                    Some('<') => {
                        // Switch to the script data double escaped less-than sign state.
                        self.switch_to(TokenizerState::ScriptDataDoubleEscapedLessThanSign);

                        // Emit a U+003C LESS-THAN SIGN character token.
                        self.emit(Token::Character('<'));
                    },
                    Some('>') => {
                        // Switch to the script data state.
                        self.switch_to(TokenizerState::ScriptData);

                        // Emit a U+003E GREATER-THAN SIGN character token.
                        self.emit(Token::Character('>'));
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        self.parse_error(HtmlParseError::UnexpectedNullCharacter);

                        // Switch to the script data double escaped state.
                        self.switch_to(TokenizerState::ScriptDataDoubleEscaped);

                        // Emit a U+FFFD REPLACEMENT CHARACTER character token.
                        self.emit(Token::Character(UNICODE_REPLACEMENT));
                    },
                    Some(c) => {
                        // Switch to the script data double escaped state.
                        self.switch_to(TokenizerState::ScriptDataDoubleEscaped);

                        // Emit the current input character as a character token.
                        self.emit(Token::Character(c));
                    },
                    None => {
                        // This is an eof-in-script-html-comment-like-text parse error.
                        self.parse_error(HtmlParseError::EOFInScriptHtmlCommentLikeText);

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-double-escaped-less-than-sign-state
            TokenizerState::ScriptDataDoubleEscapedLessThanSign => {
                // Consume the next input character:
                match self.read_next() {
                    Some('/') => {
                        // Set the temporary buffer to the empty string.
                        self.buffer.clear();

                        // Switch to the script data double escape end state.
                        self.switch_to(TokenizerState::ScriptDataDoubleEscapeEnd);

                        // Emit a U+002F SOLIDUS character token.
                        self.emit(Token::Character('/'));
                    },
                    _ => {
                        // Reconsume in the script data double escaped state.
                        self.reconsume_in(TokenizerState::ScriptDataDoubleEscaped);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-double-escape-end-state
            TokenizerState::ScriptDataDoubleEscapeEnd => {
                // Consume the next input character:
                match self.read_next() {
                    Some(c @ ('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' | '/' | '>')) => {
                        // If the temporary buffer is the string "script",
                        if self.buffer == "script" {
                            // then switch to the script data escaped state.
                            self.switch_to(TokenizerState::ScriptDataEscaped);
                        } else {
                            // Otherwise, switch to the script data double escaped state.
                            self.switch_to(TokenizerState::ScriptDataDoubleEscaped);
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
                        self.reconsume_in(TokenizerState::ScriptDataDoubleEscaped);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#before-attribute-name-state
            TokenizerState::BeforeAttributeName => {
                // Consume the next input character:
                match self.read_next() {
                    Some(TAB | LINE_FEED | FORM_FEED | SPACE) => {}, // Ignore the character.
                    Some('/' | '>') | None => {
                        // Reconsume in the after attribute name state.
                        self.reconsume_in(TokenizerState::AfterAttributeName);
                    },
                    Some('=') => {
                        // This is an unexpected-equals-sign-before-attribute-name parse error.
                        self.parse_error(HtmlParseError::UnexpectedEqualsSignBeforeAttributeName);

                        // Start a new attribute in the current tag token.
                        // Set that attribute's name to the current input character, and its value to the empty string.
                        self.current_tag.start_a_new_attribute();
                        self.current_tag.current_attribute_name.push('=');

                        // Switch to the attribute name state.
                        self.switch_to(TokenizerState::AttributeName);
                    },
                    _ => {
                        // Start a new attribute in the current tag token. Set that attribute name
                        // and value to the empty string.
                        self.current_tag.start_a_new_attribute();

                        // Reconsume in the attribute name state.
                        self.reconsume_in(TokenizerState::AttributeName);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#attribute-name-state
            TokenizerState::AttributeName => {
                // TODO: when leaving the AttributeName, we need to check
                // for duplicate attribute names.
                // https://html.spec.whatwg.org/multipage/parsing.html#attribute-name-state
                //
                // Consume the next input character:
                match self.read_next() {
                    Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' | '/' | '>') | None => {
                        // Reconsume in the after attribute name state.
                        self.reconsume_in(TokenizerState::AfterAttributeName);
                    },
                    Some('=') => {
                        // Switch to the before attribute value state.
                        self.switch_to(TokenizerState::BeforeAttributeValue);
                    },
                    Some(c @ 'A'..='Z') => {
                        // Append the lowercase version of the current input character (add 0x0020
                        // to the character's code point) to the current attribute's name.
                        self.current_tag
                            .current_attribute_name
                            .push(c.to_ascii_lowercase());
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        self.parse_error(HtmlParseError::UnexpectedNullCharacter);

                        // Append a U+FFFD REPLACEMENT CHARACTER character to the current attribute's name.
                        self.current_tag
                            .current_attribute_name
                            .push(UNICODE_REPLACEMENT);
                    },
                    Some(c @ ('"' | '\'' | '<')) => {
                        // This is an unexpected-character-in-attribute-name parse error.
                        self.parse_error(HtmlParseError::UnexpectedCharacterInAttributeName);

                        // Treat it as per the "anything else" entry below.
                        self.current_tag.current_attribute_name.push(c);
                    },
                    Some(c) => {
                        // Append the current input character to the current attribute's name.
                        self.current_tag.current_attribute_name.push(c);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#after-attribute-name-state
            TokenizerState::AfterAttributeName => {
                // Consume the next input character:
                match self.read_next() {
                    Some(TAB | LINE_FEED | FORM_FEED | SPACE) => {}, // Ignore the character.
                    Some('/') => {
                        // Switch to the self-closing start tag state.
                        self.switch_to(TokenizerState::SelfClosingStartTag);
                    },
                    Some('=') => {
                        // Switch to the before attribute value state.
                        self.switch_to(TokenizerState::BeforeAttributeValue);
                    },
                    Some('>') => {
                        // Switch to the data state. Emit the current tag token.
                        self.switch_to(TokenizerState::Data);
                        self.emit_current_tag_token();
                    },
                    Some(_) => {
                        // Start a new attribute in the current tag token.
                        // Set that attribute name and value to the empty string.
                        self.current_tag.start_a_new_attribute();

                        // Reconsume in the attribute name state.
                        self.reconsume_in(TokenizerState::AttributeName);
                    },
                    None => {
                        // This is an eof-in-tag parse error.
                        self.parse_error(HtmlParseError::EOFInTag);

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#before-attribute-value-state
            TokenizerState::BeforeAttributeValue => {
                // Consume the next input character:
                match self.read_next() {
                    Some(TAB | LINE_FEED | FORM_FEED | SPACE) => {}, // Ignore the character.
                    Some('"') => {
                        // Switch to the attribute value (double-quoted) state.
                        self.switch_to(TokenizerState::AttributeValueDoublequoted);
                    },
                    Some('\'') => {
                        // Switch to the attribute value (single-quoted) state.
                        self.switch_to(TokenizerState::AttributeValueSinglequoted);
                    },
                    Some('>') => {
                        // This is a missing-attribute-value parse error.
                        self.parse_error(HtmlParseError::MissingAttributeValue);

                        // Switch to the data state.
                        self.switch_to(TokenizerState::Data);

                        // Emit the current tag token.
                        self.emit_current_tag_token();
                    },
                    _ => {
                        // Reconsume in the attribute value (unquoted) state.
                        self.reconsume_in(TokenizerState::AttributeValueUnquoted);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(double-quoted)-state
            TokenizerState::AttributeValueDoublequoted => {
                // Consume the next input character:
                match self.read_next() {
                    Some('"') => {
                        // Switch to the after attribute value (quoted) state.
                        self.switch_to(TokenizerState::AfterAttributeValueQuoted);
                    },
                    Some('&') => {
                        // Set the return state to the attribute value (double-quoted) state.
                        self.return_state = Some(TokenizerState::AttributeValueDoublequoted);

                        // Switch to the character reference state.
                        self.switch_to(TokenizerState::CharacterReference);
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        self.parse_error(HtmlParseError::UnexpectedNullCharacter);

                        // Append a U+FFFD REPLACEMENT CHARACTER character to the current attribute's value.
                        self.current_tag
                            .current_attribute_value
                            .push(UNICODE_REPLACEMENT);
                    },
                    Some(c) => {
                        // Append the current input character to the current attribute's value.
                        self.current_tag.current_attribute_value.push(c);
                    },
                    None => {
                        // This is an eof-in-tag parse error.
                        self.parse_error(HtmlParseError::EOFInTag);

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(single-quoted)-state
            TokenizerState::AttributeValueSinglequoted => {
                // Consume the next input character:
                match self.read_next() {
                    Some('\'') => {
                        // Switch to the after attribute value (quoted) state.
                        self.switch_to(TokenizerState::AfterAttributeValueQuoted);
                    },
                    Some('&') => {
                        // Set the return state to the attribute value (single-quoted) state.
                        self.return_state = Some(TokenizerState::AttributeValueSinglequoted);

                        // Switch to the character reference state.
                        self.switch_to(TokenizerState::CharacterReference);
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        self.parse_error(HtmlParseError::UnexpectedNullCharacter);

                        // Append a U+FFFD REPLACEMENT CHARACTER character to the current attribute's value.
                        self.current_tag
                            .current_attribute_value
                            .push(UNICODE_REPLACEMENT);
                    },
                    Some(c) => {
                        // Append the current input character to the current attribute's value.
                        self.current_tag.current_attribute_value.push(c);
                    },
                    None => {
                        // This is an eof-in-tag parse error.
                        self.parse_error(HtmlParseError::EOFInTag);

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(unquoted)-state
            TokenizerState::AttributeValueUnquoted => {
                // Consume the next input character:
                match self.read_next() {
                    Some(TAB | LINE_FEED | FORM_FEED | SPACE) => {
                        // Switch to the before attribute name state.
                        self.switch_to(TokenizerState::BeforeAttributeName);
                    },
                    Some('&') => {
                        // Set the return state to the attribute value (unquoted) state.
                        self.return_state = Some(TokenizerState::AttributeValueUnquoted);

                        // Switch to the character reference state.
                        self.switch_to(TokenizerState::CharacterReference);
                    },
                    Some('>') => {
                        // Switch to the data state. Emit the current tag token.
                        self.switch_to(TokenizerState::Data);
                        self.emit_current_tag_token();
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        // Append a U+FFFD REPLACEMENT CHARACTER character to the current attribute's value.
                        self.current_tag
                            .current_attribute_value
                            .push(UNICODE_REPLACEMENT);
                    },
                    Some(c @ ('"' | '\'' | '<' | '=' | '`')) => {
                        // This is an unexpected-character-in-unquoted-attribute-value parse error.
                        self.parse_error(
                            HtmlParseError::UnexpectedCharacterInUnquotedAttributeValue,
                        );

                        // Treat it as per the "anything else" entry below.
                        self.current_tag.current_attribute_value.push(c);
                    },
                    Some(c) => {
                        // Append the current input character to the current attribute's value.
                        self.current_tag.current_attribute_value.push(c);
                    },
                    None => {
                        // This is an eof-in-tag parse error.
                        self.parse_error(HtmlParseError::EOFInTag);

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#after-attribute-value-(quoted)-state
            TokenizerState::AfterAttributeValueQuoted => {
                // Consume the next input character:
                match self.read_next() {
                    Some(TAB | LINE_FEED | FORM_FEED | SPACE) => {
                        // Switch to the before attribute name state.
                        self.switch_to(TokenizerState::BeforeAttributeName);
                    },
                    Some('/') => {
                        // Switch to the self-closing start tag state.
                        self.switch_to(TokenizerState::SelfClosingStartTag);
                    },
                    Some('>') => {
                        // Switch to the data state. Emit the current tag token.
                        self.switch_to(TokenizerState::Data);
                        self.emit_current_tag_token();
                    },
                    Some(_) => {
                        // This is a missing-whitespace-between-attributes parse error.
                        self.parse_error(HtmlParseError::MissingWhitespaceBetweenAttributes);

                        // Reconsume in the before attribute name state.
                        self.reconsume_in(TokenizerState::BeforeAttributeName);
                    },
                    None => {
                        // This is an eof-in-tag parse error.
                        self.parse_error(HtmlParseError::EOFInTag);

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#self-closing-start-tag-state
            TokenizerState::SelfClosingStartTag => {
                // Consume the next input character:
                match self.read_next() {
                    Some('>') => {
                        // Set the self-closing flag of the current tag token.
                        self.current_tag.is_self_closing = true;

                        // Switch to the data state.
                        self.switch_to(TokenizerState::Data);

                        // Emit the current tag token.
                        self.emit_current_tag_token();
                    },
                    Some(_) => {
                        // This is an unexpected-solidus-in-tag parse error.
                        self.parse_error(HtmlParseError::UnexpectedSolidusInTag);

                        // Reconsume in the before attribute name state.
                        self.reconsume_in(TokenizerState::BeforeAttributeName);
                    },
                    None => {
                        // This is an eof-in-tag parse error.
                        self.parse_error(HtmlParseError::EOFInTag);

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#bogus-comment-state
            TokenizerState::BogusComment => {
                // Consume the next input character:
                match self.read_next() {
                    Some('>') => {
                        // Switch to the data state.
                        self.switch_to(TokenizerState::Data);

                        // Emit the current comment token.
                        self.emit_current_comment_token();
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        self.parse_error(HtmlParseError::UnexpectedNullCharacter);

                        // Append a U+FFFD REPLACEMENT CHARACTER character to the comment token's data.
                        self.current_comment.push(UNICODE_REPLACEMENT);
                    },
                    Some(c) => {
                        // Append the current input character to the comment token's data.
                        self.current_comment.push(c);
                    },
                    None => {
                        // Emit the comment.
                        self.emit_current_comment_token();

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#markup-declaration-open-state
            TokenizerState::MarkupDeclarationOpen => {
                // If the next few characters are:
                if self.source.remaining().starts_with("--") {
                    // Consume those two characters, create a comment token whose data is the empty
                    // string, and switch to the comment start state.
                    let _ = self.source.advance_by(2);
                    self.current_comment.clear();
                    self.switch_to(TokenizerState::CommentStart);
                } else if self.source.remaining()[..7].eq_ignore_ascii_case("DOCTYPE") {
                    // Consume those characters and switch to the DOCTYPE state.
                    let _ = self.source.advance_by(7);
                    self.switch_to(TokenizerState::DOCTYPE);
                } else if self.source.remaining().starts_with("[CDATA[") {
                    // Consume those characters. If there is an adjusted current node and it is not
                    // an element in the HTML namespace, then switch to the CDATA section state.
                    // Otherwise, this is a cdata-in-html-content parse error. Create a comment
                    // token whose data is the "[CDATA[" string. Switch to the bogus comment state.
                    let _ = self.source.advance_by(7);
                    todo!();
                } else {
                    // This is an incorrectly-opened-comment parse error.
                    self.parse_error(HtmlParseError::IncorrectlyOpenedComment);

                    // Create a comment token whose data is the empty string.
                    self.current_comment.clear();

                    // Switch to the bogus comment state (don't consume anything in the current state).
                    self.switch_to(TokenizerState::BogusComment);
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#comment-start-state
            TokenizerState::CommentStart => {
                // Consume the next input character:
                match self.read_next() {
                    Some('-') => {
                        // Switch to the comment start dash state.
                        self.switch_to(TokenizerState::CommentStartDash);
                    },
                    Some('>') => {
                        // This is an abrupt-closing-of-empty-comment parse error.
                        self.parse_error(HtmlParseError::AbruptClosingOfEmptyComment);

                        // Switch to the data state.
                        self.switch_to(TokenizerState::Data);

                        // Emit the current comment token.
                        self.emit_current_comment_token();
                    },
                    _ => {
                        // Reconsume in the comment state.
                        self.reconsume_in(TokenizerState::Comment);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#comment-start-dash-state
            TokenizerState::CommentStartDash => {
                // Consume the next input character:
                match self.read_next() {
                    Some('-') => {
                        // Switch to the comment end state.
                        self.switch_to(TokenizerState::CommentEnd);
                    },
                    Some('>') => {
                        // This is an abrupt-closing-of-empty-comment parse error.
                        self.parse_error(HtmlParseError::AbruptClosingOfEmptyComment);

                        // Switch to the data state.
                        self.switch_to(TokenizerState::Data);

                        // Emit the current comment token.
                        self.emit_current_comment_token();
                    },
                    Some(_) => {
                        // Append a U+002D HYPHEN-MINUS character (-) to the comment token's data.
                        self.current_comment.push('-');

                        // Reconsume in the comment state.
                        self.reconsume_in(TokenizerState::Comment);
                    },
                    None => {
                        // This is an eof-in-comment parse error.
                        self.parse_error(HtmlParseError::EOFInComment);

                        // Emit the current comment token.
                        self.emit_current_comment_token();

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#comment-state
            TokenizerState::Comment => {
                // Consume the next input character:
                match self.read_next() {
                    Some('>') => {
                        // Append the current input character to the comment token's data.
                        self.current_comment.push('<');

                        // Switch to the comment less-than sign state.
                        self.switch_to(TokenizerState::CommentLessThanSign);
                    },
                    Some('-') => {
                        // Switch to the comment end dash state.
                        self.switch_to(TokenizerState::CommentEndDash);
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        self.parse_error(HtmlParseError::UnexpectedNullCharacter);

                        // Append a U+FFFD REPLACEMENT CHARACTER character to the comment token's data.
                        self.current_comment.push(UNICODE_REPLACEMENT);
                    },
                    Some(c) => {
                        // Append the current input character to the comment token's data.
                        self.current_comment.push(c);
                    },
                    None => {
                        // This is an eof-in-comment parse error.
                        self.parse_error(HtmlParseError::EOFInComment);

                        // Emit the current comment token.
                        self.emit_current_comment_token();

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#comment-less-than-sign-state
            TokenizerState::CommentLessThanSign => {
                // Consume the next input character:
                match self.read_next() {
                    Some('!') => {
                        // Append the current input character to the comment token's data.
                        self.current_comment.push('!');

                        // Switch to the comment less-than sign bang state.
                        self.switch_to(TokenizerState::CommentLessThanSignBang);
                    },
                    Some('<') => {
                        // Append the current input character to the comment token's data.
                        self.current_comment.push('<');
                    },
                    _ => {
                        // Reconsume in the comment state.
                        self.reconsume_in(TokenizerState::Comment);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#comment-less-than-sign-bang-state
            TokenizerState::CommentLessThanSignBang => {
                // Consume the next input character:
                match self.read_next() {
                    Some('-') => {
                        // Switch to the comment less-than sign bang dash state.
                        self.switch_to(TokenizerState::CommentLessThanSignBangDash);
                    },
                    _ => {
                        // Reconsume in the comment state.
                        self.reconsume_in(TokenizerState::Comment);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#comment-less-than-sign-bang-dash-state
            TokenizerState::CommentLessThanSignBangDash => {
                // Consume the next input character:
                match self.read_next() {
                    Some('-') => {
                        // Switch to the comment less-than sign bang dash dash state.
                        self.switch_to(TokenizerState::CommentLessThanSignBangDashDash);
                    },
                    _ => {
                        // Reconsume in the comment end dash state.
                        self.reconsume_in(TokenizerState::CommentEndDash);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#comment-less-than-sign-bang-dash-dash-state
            TokenizerState::CommentLessThanSignBangDashDash => {
                // Consume the next input character:
                match self.read_next() {
                    Some('>') | None => {
                        // Reconsume in the comment end state.
                        self.reconsume_in(TokenizerState::CommentEnd);
                    },
                    Some(_) => {
                        // This is a nested-comment parse error.
                        self.parse_error(HtmlParseError::NestedComment);

                        // Reconsume in the comment end state.
                        self.reconsume_in(TokenizerState::CommentEnd);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#comment-end-dash-state
            TokenizerState::CommentEndDash => {
                // Consume the next input character:
                match self.read_next() {
                    Some('-') => {
                        // Switch to the comment end state.
                        self.switch_to(TokenizerState::CommentEnd);
                    },
                    Some(_) => {
                        // Append a U+002D HYPHEN-MINUS character (-) to the comment token's data.
                        self.current_comment.push('-');

                        // Reconsume in the comment state.
                        self.reconsume_in(TokenizerState::Comment);
                    },
                    None => {
                        // This is an eof-in-comment parse error.
                        self.parse_error(HtmlParseError::EOFInComment);

                        // Emit the current comment token.
                        self.emit_current_comment_token();

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#comment-end-state
            TokenizerState::CommentEnd => {
                // Consume the next input character:
                match self.read_next() {
                    Some('>') => {
                        // Switch to the data state.
                        self.switch_to(TokenizerState::Data);

                        // Emit the current comment token.
                        self.emit_current_comment_token();
                    },
                    Some('!') => {
                        // Switch to the comment end bang state.
                        self.switch_to(TokenizerState::CommentEndBang);
                    },
                    Some('-') => {
                        // Append a U+002D HYPHEN-MINUS character (-) to the comment token's data.
                        self.current_comment.push('-');
                    },
                    Some(_) => {
                        // Append two U+002D HYPHEN-MINUS characters (-) to the comment token's
                        // data.
                        self.current_comment.push('-');
                        self.current_comment.push('-');

                        // Reconsume in the comment state.
                        self.reconsume_in(TokenizerState::Comment);
                    },
                    None => {
                        // This is an eof-in-comment parse error.
                        self.parse_error(HtmlParseError::EOFInComment);

                        // Emit the current comment token.
                        self.emit_current_comment_token();

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#comment-end-bang-state
            TokenizerState::CommentEndBang => {
                // Consume the next input character:
                match self.read_next() {
                    Some('-') => {
                        // Append two U+002D HYPHEN-MINUS characters (-) and a U+0021 EXCLAMATION
                        // MARK character (!) to the comment token's data.
                        self.current_comment.push('-');
                        self.current_comment.push('-');
                        self.current_comment.push('!');

                        // Switch to the comment end dash state.
                        self.switch_to(TokenizerState::CommentEndDash);
                    },
                    Some('>') => {
                        // This is an incorrectly-closed-comment parse error.
                        self.parse_error(HtmlParseError::IncorrectlyClosedComment);

                        // Switch to the data state.
                        self.switch_to(TokenizerState::Data);

                        // Emit the current comment token.
                        self.emit_current_comment_token();
                    },
                    Some(_) => {
                        // Append two U+002D HYPHEN-MINUS characters (-) and a U+0021 EXCLAMATION
                        // MARK character (!) to the comment token's data.
                        self.current_comment.push('-');
                        self.current_comment.push('-');
                        self.current_comment.push('!');

                        // Reconsume in the comment state.
                        self.reconsume_in(TokenizerState::Comment);
                    },
                    None => {
                        // This is an eof-in-comment parse error.
                        self.parse_error(HtmlParseError::EOFInComment);

                        // Emit the current comment token.
                        self.emit_current_comment_token();

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#doctype-state
            TokenizerState::DOCTYPE => {
                // Consume the next input character:
                match self.read_next() {
                    Some(TAB | LINE_FEED | FORM_FEED | SPACE) => {
                        // Switch to the before DOCTYPE name state.
                        self.switch_to(TokenizerState::BeforeDOCTYPEName);
                    },
                    Some('>') => {
                        // Reconsume in the before DOCTYPE name state.
                        self.reconsume_in(TokenizerState::BeforeDOCTYPEName);
                    },
                    Some(_) => {
                        // This is a missing-whitespace-before-doctype-name parse error.
                        self.parse_error(HtmlParseError::MissingWhitespaceBeforeDoctypeName);

                        // Reconsume in the before DOCTYPE name state.
                        self.reconsume_in(TokenizerState::BeforeDOCTYPEName);
                    },
                    None => {
                        // This is an eof-in-doctype parse error.
                        self.parse_error(HtmlParseError::EOFInDoctype);

                        // Create a new DOCTYPE token.
                        self.current_token.create_doctype();

                        // Set its force-quirks flag to on.
                        self.current_token.set_force_quirks();

                        // Emit the current token.
                        self.emit_current_token();

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#before-doctype-name-state
            TokenizerState::BeforeDOCTYPEName => {
                // Note: this code potentially emits tokens *without* modifying self.current_token!

                // Consume the next input character:
                match self.read_next() {
                    Some(TAB | LINE_FEED | FORM_FEED | SPACE) => {}, // Ignore the character.
                    Some(c @ 'A'..='Z') => {
                        // Create a new DOCTYPE token.
                        self.current_token.create_doctype();

                        // Set the token's name to the lowercase version of the current input character (add 0x0020 to the
                        // character's code point).
                        self.current_token
                            .append_to_doctype_name(c.to_ascii_lowercase());

                        // Switch to the DOCTYPE name state.
                        self.switch_to(TokenizerState::DOCTYPEName);
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        self.parse_error(HtmlParseError::UnexpectedNullCharacter);

                        // Create a new DOCTYPE token.
                        self.current_token.create_doctype();

                        // Set the token's name to a U+FFFD REPLACEMENT CHARACTER character.
                        self.current_token
                            .append_to_doctype_name(UNICODE_REPLACEMENT);

                        // Switch to the DOCTYPE name state.
                        self.switch_to(TokenizerState::DOCTYPEName);
                    },
                    Some('>') => {
                        // This is a missing-doctype-name parse error.
                        self.parse_error(HtmlParseError::MissingDoctypeName);

                        // Create a new DOCTYPE token.
                        self.current_token.create_doctype();

                        // Set its force-quirks flag to on.
                        self.current_token.set_force_quirks();

                        // Switch to the data state.
                        self.switch_to(TokenizerState::Data);

                        // Emit the current token.
                        self.emit_current_token();
                    },
                    Some(c) => {
                        // Create a new DOCTYPE token.
                        self.current_token.create_doctype();

                        // Set the token's name to the current input character.
                        self.current_token.append_to_doctype_name(c);

                        // Switch to the DOCTYPE name state.
                        self.switch_to(TokenizerState::DOCTYPEName);
                    },
                    None => {
                        // This is an eof-in-doctype parse error.
                        self.parse_error(HtmlParseError::EOFInDoctype);

                        // Create a new DOCTYPE token.
                        self.current_token.create_doctype();

                        // Set its force-quirks flag to on.
                        self.current_token.set_force_quirks();

                        // Emit the current token.
                        self.emit_current_token();

                        // Emit an end-of-file token.
                        self.emit(Token::EOF)
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#doctype-name-state
            TokenizerState::DOCTYPEName => {
                // Consume the next input character:
                match self.read_next() {
                    Some(TAB | LINE_FEED | FORM_FEED | SPACE) => {
                        // Switch to the after DOCTYPE name state.
                        self.switch_to(TokenizerState::AfterDOCTYPEName);
                    },
                    Some('>') => {
                        // Switch to the data state. Emit the current DOCTYPE token.
                        self.switch_to(TokenizerState::Data);
                        self.emit_current_token();
                    },
                    Some(c @ 'A'..='Z') => {
                        // Append the lowercase version of the current input character (add 0x0020
                        // to the character's code point) to the current DOCTYPE token's name.
                        self.current_token
                            .append_to_doctype_name(c.to_ascii_lowercase());
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        self.parse_error(HtmlParseError::UnexpectedNullCharacter);

                        // Append a U+FFFD REPLACEMENT CHARACTER character to the current DOCTYPE token's name.
                        self.current_token
                            .append_to_doctype_name(UNICODE_REPLACEMENT);
                    },
                    Some(c) => {
                        // Append the current input character to the current DOCTYPE token's name.
                        self.current_token.append_to_doctype_name(c);
                    },
                    None => {
                        // This is an eof-in-doctype parse error.
                        self.parse_error(HtmlParseError::EOFInDoctype);

                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.current_token.set_force_quirks();

                        // Emit the current DOCTYPE token.
                        self.emit_current_token();

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#after-doctype-name-state
            TokenizerState::AfterDOCTYPEName => {
                // Consume the next input character:
                match self.read_next() {
                    Some(TAB | LINE_FEED | FORM_FEED | SPACE) => {}, // Ignore the character.
                    Some('>') => {
                        // Switch to the data state. Emit the current DOCTYPE token.
                        self.switch_to(TokenizerState::Data);
                        self.emit_current_token();
                    },
                    None => {
                        // This is an eof-in-doctype parse error.
                        self.parse_error(HtmlParseError::EOFInDoctype);

                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.current_token.set_force_quirks();

                        // Emit the current DOCTYPE token.
                        self.emit_current_token();

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                    Some(_) => {
                        // NOTE: the next tests include the current input character,
                        //       so we go back one and effectively reconsume it.
                        self.source.go_back();

                        // If the six characters starting from the current input character are
                        // an ASCII case-insensitive match for the word "PUBLIC",
                        if self.source.remaining().len() > 6 {
                            let next_six_chars = &self.source.remaining()[..6];
                            if next_six_chars.eq_ignore_ascii_case("PUBLIC") {
                                // then consume those characters
                                let _ = self.source.advance_by(6);

                                // and switch to the after DOCTYPE public keyword state.
                                self.switch_to(TokenizerState::AfterDOCTYPEPublicKeyword);
                                return;
                            }
                            // Otherwise, if the six characters starting from the current input
                            // character are an ASCII case-insensitive match for the word
                            // "SYSTEM",
                            else if next_six_chars.eq_ignore_ascii_case("SYSTEM") {
                                // then consume those characters
                                let _ = self.source.advance_by(6);

                                // and switch to the after DOCTYPE system keyword state.
                                self.switch_to(TokenizerState::AfterDOCTYPESystemKeyword);
                                return;
                            }
                        }

                        // Otherwise, this is an invalid-character-sequence-after-doctype-name parse error.
                        self.parse_error(HtmlParseError::InvalidCharacterSequenceAfterDoctypeName);

                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.current_token.set_force_quirks();

                        // Reconsume in the bogus DOCTYPE state.
                        self.current_token.set_force_quirks();

                        // Note: we reconsume, but because we already decremented
                        // self.ptr (above) we don't need to do it again
                        self.switch_to(TokenizerState::BogusDOCTYPE);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#after-doctype-public-keyword-state
            TokenizerState::AfterDOCTYPEPublicKeyword => {
                // Consume the next input character:
                match self.read_next() {
                    Some(TAB | LINE_FEED | FORM_FEED | SPACE) => {
                        // Switch to the before DOCTYPE public identifier state.
                        self.switch_to(TokenizerState::BeforeDOCTYPEPublicIdentifier);
                    },
                    Some('"') => {
                        // This is a missing-whitespace-after-doctype-public-keyword parse error.
                        self.parse_error(
                            HtmlParseError::MissingWhitespaceAfterDoctypePublicKeyword,
                        );

                        // Set the current DOCTYPE token's public identifier to the empty string
                        // (not missing),
                        self.current_token.init_doctype_public_ident();

                        // then switch to the DOCTYPE public identifier (double-quoted) state.
                        self.switch_to(TokenizerState::DOCTYPEPublicIdentifierDoublequoted);
                    },
                    Some('\'') => {
                        // This is a missing-whitespace-after-doctype-public-keyword parse error.
                        self.parse_error(
                            HtmlParseError::MissingWhitespaceAfterDoctypePublicKeyword,
                        );

                        // Set the current DOCTYPE token's public identifier to the empty string
                        // (not missing),
                        self.current_token.init_doctype_public_ident();

                        // then switch to the DOCTYPE public identifier (single-quoted) state.
                        self.switch_to(TokenizerState::DOCTYPEPublicIdentifierSinglequoted);
                    },
                    Some('>') => {
                        // This is a missing-doctype-public-identifier parse error.
                        self.parse_error(HtmlParseError::MissingDoctypePublicIdentifier);

                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.current_token.set_force_quirks();

                        // Switch to the data state.
                        self.switch_to(TokenizerState::Data);

                        // Emit the current DOCTYPE token.
                        self.emit_current_token();
                    },
                    Some(_) => {
                        // This is a missing-quote-before-doctype-public-identifier parse error.
                        self.parse_error(HtmlParseError::MissingQuoteBeforeDoctypePublicIdentifier);

                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.current_token.set_force_quirks();

                        // Reconsume in the bogus DOCTYPE state.
                        self.reconsume_in(TokenizerState::BogusDOCTYPE);
                    },
                    None => {
                        // This is an eof-in-doctype parse error.
                        self.parse_error(HtmlParseError::EOFInDoctype);

                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.current_token.set_force_quirks();

                        // Emit the current DOCTYPE token.
                        self.emit_current_token();

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#before-doctype-public-identifier-state
            TokenizerState::BeforeDOCTYPEPublicIdentifier => {
                // Consume the next input character:
                match self.read_next() {
                    Some(TAB | LINE_FEED | FORM_FEED | SPACE) => {}, // Ignore the character.
                    Some('"') => {
                        // Set the current DOCTYPE token's public identifier to the empty string
                        // (not missing),
                        self.current_token.init_doctype_public_ident();

                        // then switch to the DOCTYPE public identifier (double-quoted) state.
                        self.switch_to(TokenizerState::DOCTYPEPublicIdentifierDoublequoted);
                    },
                    Some('\'') => {
                        // Set the current DOCTYPE token's public identifier to the empty string
                        // (not missing),
                        self.current_token.init_doctype_public_ident();

                        // then switch to the DOCTYPE public identifier (single-quoted) state.
                        self.switch_to(TokenizerState::DOCTYPEPublicIdentifierSinglequoted);
                    },
                    Some('>') => {
                        // This is a missing-doctype-public-identifier parse error.
                        self.parse_error(HtmlParseError::MissingDoctypePublicIdentifier);

                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.current_token.set_force_quirks();

                        // Switch to the data state.
                        self.switch_to(TokenizerState::Data);

                        // Emit the current DOCTYPE token.
                        self.emit_current_token();
                    },
                    Some(_) => {
                        // This is a missing-quote-before-doctype-public-identifier parse error.
                        self.parse_error(HtmlParseError::MissingQuoteBeforeDoctypePublicIdentifier);

                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.current_token.set_force_quirks();

                        // Reconsume in the bogus DOCTYPE state.
                        self.reconsume_in(TokenizerState::BogusDOCTYPE);
                    },
                    None => {
                        // This is an eof-in-doctype parse error.
                        self.parse_error(HtmlParseError::EOFInDoctype);

                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.current_token.set_force_quirks();

                        // Emit the current DOCTYPE token.
                        self.emit_current_token();

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#doctype-public-identifier-(double-quoted)-state
            TokenizerState::DOCTYPEPublicIdentifierDoublequoted => {
                // Consume the next input character:
                match self.read_next() {
                    Some('"') => {
                        // Switch to the after DOCTYPE public identifier state.
                        self.switch_to(TokenizerState::AfterDOCTYPEPublicIdentifier);
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        self.parse_error(HtmlParseError::UnexpectedNullCharacter);

                        // Append a U+FFFD REPLACEMENT CHARACTER character to the current DOCTYPE token's public
                        // identifier.
                        self.current_token
                            .append_to_doctype_public_ident(UNICODE_REPLACEMENT);
                    },
                    Some('>') => {
                        // This is an abrupt-doctype-public-identifier parse error.
                        self.parse_error(HtmlParseError::AbruptDoctypePublicIdentifier);

                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.current_token.set_force_quirks();

                        // Switch to the data state.
                        self.switch_to(TokenizerState::Data);

                        // Emit the current DOCTYPE token.
                        self.emit_current_token();
                    },
                    Some(c) => {
                        // Append the current input character to the current DOCTYPE token's
                        // public identifier.
                        self.current_token.append_to_doctype_public_ident(c);
                    },
                    None => {
                        // This is an eof-in-doctype parse error.
                        self.parse_error(HtmlParseError::EOFInDoctype);

                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.current_token.set_force_quirks();

                        // Emit the current DOCTYPE token.
                        self.emit_current_token();

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#doctype-public-identifier-(single-quoted)-state
            TokenizerState::DOCTYPEPublicIdentifierSinglequoted => {
                // Consume the next input character:
                match self.read_next() {
                    Some('\'') => {
                        // Switch to the after DOCTYPE public identifier state.
                        self.switch_to(TokenizerState::AfterDOCTYPEPublicIdentifier);
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        self.parse_error(HtmlParseError::UnexpectedNullCharacter);

                        // Append a U+FFFD REPLACEMENT CHARACTER character to the current DOCTYPE token's public
                        // identifier.
                        self.current_token
                            .append_to_doctype_public_ident(UNICODE_REPLACEMENT);
                    },
                    Some('>') => {
                        // This is an abrupt-doctype-public-identifier parse error.
                        self.parse_error(HtmlParseError::AbruptDoctypePublicIdentifier);

                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.current_token.set_force_quirks();

                        // Switch to the data state.
                        self.switch_to(TokenizerState::Data);

                        // Emit the current DOCTYPE token.
                        self.emit_current_token();
                    },
                    Some(c) => {
                        // Append the current input character to the current DOCTYPE token's public
                        // identifier.
                        self.current_token.append_to_doctype_public_ident(c);
                    },
                    None => {
                        // This is an eof-in-doctype parse error.
                        self.parse_error(HtmlParseError::EOFInDoctype);

                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.current_token.set_force_quirks();

                        // Emit the current DOCTYPE token.
                        self.emit_current_token();

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#after-doctype-public-identifier-state
            TokenizerState::AfterDOCTYPEPublicIdentifier => {
                // Consume the next input character:
                match self.read_next() {
                    Some(TAB | LINE_FEED | FORM_FEED | SPACE) => {
                        // Switch to the between DOCTYPE public and system identifiers state.
                        self.switch_to(TokenizerState::BetweenDOCTYPEPublicAndSystemIdentifiers);
                    },
                    Some('>') => {
                        // Switch to the data state. Emit the current DOCTYPE token.
                        self.switch_to(TokenizerState::Data);
                        self.emit_current_token();
                    },
                    Some('"') => {
                        // This is a
                        // missing-whitespace-between-doctype-public-and-system-identifiers parse
                        // error.
                        self.parse_error(HtmlParseError::MissingWhitespaceBetweenDoctypePublicAndSystemIdentifier);

                        // Set the current DOCTYPE token's system identifier to the empty
                        // string (not missing),
                        self.current_token.init_doctype_system_ident();

                        // then switch to the DOCTYPE system identifier (double-quoted) state.
                        self.switch_to(TokenizerState::DOCTYPESystemIdentifierDoublequoted);
                    },
                    Some('\'') => {
                        // This is a missing-whitespace-between-doctype-public-and-system-identifiers
                        // parse error.
                        self.parse_error(HtmlParseError::MissingWhitespaceBetweenDoctypePublicAndSystemIdentifier);

                        // Set the current DOCTYPE token's system identifier to
                        // the empty string (not missing),
                        self.current_token.init_doctype_system_ident();

                        // then switch to the DOCTYPE system identifier (single-quoted) state.
                        self.switch_to(TokenizerState::DOCTYPESystemIdentifierSinglequoted);
                    },
                    Some(_) => {
                        // This is a missing-quote-before-doctype-system-identifier parse error.
                        self.parse_error(HtmlParseError::MissingQuoteBeforeDoctypeSystemIdentifier);

                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.current_token.set_force_quirks();

                        // Reconsume in the bogus DOCTYPE state.
                        self.reconsume_in(TokenizerState::BogusDOCTYPE);
                    },
                    None => {
                        // This is an eof-in-doctype parse error.
                        self.parse_error(HtmlParseError::EOFInDoctype);

                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.current_token.set_force_quirks();

                        // Emit the current DOCTYPE token.
                        self.emit_current_token();

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#between-doctype-public-and-system-identifiers-state
            TokenizerState::BetweenDOCTYPEPublicAndSystemIdentifiers => {
                // Consume the next input character:
                match self.read_next() {
                    Some(TAB | LINE_FEED | FORM_FEED | SPACE) => {}, // Ignore the character.
                    Some('>') => {
                        // Switch to the data state. Emit the current DOCTYPE token.
                        self.switch_to(TokenizerState::Data);
                        self.emit_current_token();
                    },
                    Some('"') => {
                        // Set the current DOCTYPE token's system identifier to the empty string
                        // (not missing), then switch to the DOCTYPE system identifier
                        // (double-quoted) state.
                        self.current_token.init_doctype_system_ident();
                        self.switch_to(TokenizerState::DOCTYPESystemIdentifierDoublequoted);
                    },
                    Some('\'') => {
                        // Set the current DOCTYPE token's system identifier to the empty string
                        // (not missing), then switch to the DOCTYPE system identifier
                        // (single-quoted) state.
                        self.current_token.init_doctype_system_ident();
                        self.switch_to(TokenizerState::DOCTYPESystemIdentifierSinglequoted);
                    },
                    Some(_) => {
                        // This is a missing-quote-before-doctype-system-identifier parse error.
                        self.parse_error(HtmlParseError::MissingQuoteBeforeDoctypeSystemIdentifier);

                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.current_token.set_force_quirks();

                        // Reconsume in the bogus DOCTYPE state.
                        self.reconsume_in(TokenizerState::BogusDOCTYPE);
                    },
                    None => {
                        // This is an eof-in-doctype parse error.
                        self.parse_error(HtmlParseError::EOFInDoctype);

                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.current_token.set_force_quirks();

                        // Emit the current DOCTYPE token.
                        self.emit_current_token();

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#after-doctype-system-keyword-state
            TokenizerState::AfterDOCTYPESystemKeyword => {
                // Consume the next input character:
                match self.read_next() {
                    Some(TAB | LINE_FEED | FORM_FEED | SPACE) => {
                        // Switch to the before DOCTYPE system identifier state.
                        self.switch_to(TokenizerState::BeforeDOCTYPESystemIdentifier);
                    },
                    Some('"') => {
                        // This is a missing-whitespace-after-doctype-system-keyword parse error.
                        self.parse_error(
                            HtmlParseError::MissingWhitespaceAfterDoctypeSystemKeyword,
                        );

                        // Set the current DOCTYPE token's system identifier to the empty string
                        // (not missing), then switch to the DOCTYPE system identifier
                        // (double-quoted) state.
                        self.current_token.init_doctype_system_ident();
                        self.switch_to(TokenizerState::DOCTYPESystemIdentifierDoublequoted);
                    },
                    Some('\'') => {
                        // This is a missing-whitespace-after-doctype-system-keyword parse error.
                        self.parse_error(
                            HtmlParseError::MissingWhitespaceAfterDoctypeSystemKeyword,
                        );

                        // Set the current DOCTYPE token's system identifier to the empty string
                        // (not missing), then switch to the DOCTYPE system identifier
                        // (single-quoted) state.
                        self.current_token.init_doctype_system_ident();
                        self.switch_to(TokenizerState::DOCTYPESystemIdentifierSinglequoted);
                    },
                    Some('>') => {
                        // This is a missing-doctype-system-identifier parse error.
                        self.parse_error(HtmlParseError::MissingDoctypeSystemIdentifier);

                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.current_token.set_force_quirks();

                        // Switch to the data state.
                        self.switch_to(TokenizerState::Data);

                        // Emit the current DOCTYPE token.
                        self.emit_current_token();
                    },
                    Some(_) => {
                        // This is a missing-quote-before-doctype-system-identifier parse error.
                        self.parse_error(HtmlParseError::MissingQuoteBeforeDoctypeSystemIdentifier);

                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.current_token.set_force_quirks();

                        // Reconsume in the bogus DOCTYPE state.
                        self.reconsume_in(TokenizerState::BogusDOCTYPE);
                    },
                    None => {
                        // This is an eof-in-doctype parse error.
                        self.parse_error(HtmlParseError::EOFInDoctype);

                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.current_token.set_force_quirks();

                        // Emit the current DOCTYPE token.
                        self.emit_current_token();

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#before-doctype-system-identifier-state
            TokenizerState::BeforeDOCTYPESystemIdentifier => {
                // Consume the next input character:
                match self.read_next() {
                    Some(TAB | LINE_FEED | FORM_FEED | SPACE) => {}, //     Ignore the character.
                    Some('"') => {
                        // Set the current DOCTYPE token's system identifier to the empty string
                        // (not missing), then switch to the DOCTYPE system identifier
                        // (double-quoted) state.
                        self.current_token.init_doctype_system_ident();
                        self.switch_to(TokenizerState::DOCTYPESystemIdentifierDoublequoted);
                    },
                    Some('\'') => {
                        // Set the current DOCTYPE token's system identifier to the empty string
                        // (not missing), then switch to the DOCTYPE system identifier
                        // (single-quoted) state.
                        self.current_token.init_doctype_system_ident();
                        self.switch_to(TokenizerState::DOCTYPESystemIdentifierSinglequoted);
                    },
                    Some('>') => {
                        // This is a missing-doctype-system-identifier parse error.
                        self.parse_error(HtmlParseError::MissingDoctypeSystemIdentifier);

                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.current_token.set_force_quirks();

                        // Switch to the data state.
                        self.switch_to(TokenizerState::Data);

                        // Emit the current DOCTYPE token.
                        self.emit_current_token();
                    },
                    Some(_) => {
                        // This is a missing-quote-before-doctype-system-identifier parse error.
                        self.parse_error(HtmlParseError::MissingQuoteBeforeDoctypeSystemIdentifier);

                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.current_token.set_force_quirks();

                        // Reconsume in the bogus DOCTYPE state.
                        self.reconsume_in(TokenizerState::BogusDOCTYPE);
                    },
                    None => {
                        // This is an eof-in-doctype parse error.
                        self.parse_error(HtmlParseError::EOFInDoctype);

                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.current_token.set_force_quirks();

                        // Emit the current DOCTYPE token.
                        self.emit_current_token();

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#doctype-system-identifier-(double-quoted)-state
            TokenizerState::DOCTYPESystemIdentifierDoublequoted => {
                // Consume the next input character:
                match self.read_next() {
                    Some('"') => {
                        // Switch to the after DOCTYPE system identifier state.
                        self.switch_to(TokenizerState::AfterDOCTYPESystemIdentifier);
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        self.parse_error(HtmlParseError::UnexpectedNullCharacter);

                        // Append a U+FFFD REPLACEMENT CHARACTER character to the current
                        // DOCTYPE token's system identifier.
                        self.current_token
                            .append_to_doctype_system_ident(UNICODE_REPLACEMENT);
                    },
                    Some('>') => {
                        // This is an abrupt-doctype-system-identifier parse error.
                        self.parse_error(HtmlParseError::AbruptDoctypeSystemIdentifier);

                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.current_token.set_force_quirks();

                        // Switch to the data state.
                        self.switch_to(TokenizerState::Data);

                        // Emit the current DOCTYPE token.
                        self.emit_current_token();
                    },
                    Some(c) => {
                        // Append the current input character to the current DOCTYPE token's system
                        // identifier.
                        self.current_token.append_to_doctype_system_ident(c);
                    },
                    None => {
                        // This is an eof-in-doctype parse error.
                        self.parse_error(HtmlParseError::EOFInDoctype);

                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.current_token.set_force_quirks();

                        // Emit the current DOCTYPE token.
                        self.emit_current_token();

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#doctype-system-identifier-(single-quoted)-state
            TokenizerState::DOCTYPESystemIdentifierSinglequoted => {
                // Consume the next input character:
                match self.read_next() {
                    Some('\'') => {
                        // Switch to the after DOCTYPE system identifier state.
                        self.switch_to(TokenizerState::AfterDOCTYPESystemIdentifier);
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        self.parse_error(HtmlParseError::UnexpectedNullCharacter);

                        // Append a U+FFFD REPLACEMENT CHARACTER character to the current DOCTYPE token's system
                        // identifier.
                        self.current_token
                            .append_to_doctype_system_ident(UNICODE_REPLACEMENT);
                    },
                    Some('>') => {
                        // This is an abrupt-doctype-system-identifier parse error.
                        self.parse_error(HtmlParseError::AbruptDoctypeSystemIdentifier);

                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.current_token.set_force_quirks();

                        // Switch to the data state.
                        self.switch_to(TokenizerState::Data);

                        // Emit the current DOCTYPE token.
                        self.emit_current_token();
                    },
                    Some(c) => {
                        // Append the current input character to the current DOCTYPE token's system
                        // identifier.
                        self.current_token.append_to_doctype_system_ident(c);
                    },
                    None => {
                        // This is an eof-in-doctype parse error.
                        self.parse_error(HtmlParseError::EOFInDoctype);

                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.current_token.set_force_quirks();

                        // Emit the current DOCTYPE token.
                        self.emit_current_token();

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#after-doctype-system-identifier-state
            TokenizerState::AfterDOCTYPESystemIdentifier => {
                // Consume the next input character:
                match self.read_next() {
                    Some(TAB | LINE_FEED | FORM_FEED | SPACE) => {}, // Ignore the character.
                    Some('>') => {
                        // Switch to the data state.
                        self.switch_to(TokenizerState::Data);

                        // Emit the current DOCTYPE token.
                        self.emit_current_token();
                    },
                    Some(_) => {
                        // This is an unexpected-character-after-doctype-system-identifier parse
                        // error.
                        self.parse_error(
                            HtmlParseError::UnexpectedCharacterAfterDoctypeSystemIdentifier,
                        );

                        // Reconsume in the bogus DOCTYPE state. (This does not set the
                        // current DOCTYPE token's force-quirks flag to on.)
                        self.reconsume_in(TokenizerState::BogusDOCTYPE);
                    },
                    None => {
                        // This is an eof-in-doctype parse error.
                        self.parse_error(HtmlParseError::EOFInDoctype);

                        // Set the current DOCTYPE token's force-quirks flag to on.
                        self.current_token.set_force_quirks();

                        // Emit the current DOCTYPE token.
                        self.emit_current_token();

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#bogus-doctype-state
            TokenizerState::BogusDOCTYPE => {
                // Consume the next input character:
                match self.read_next() {
                    Some('>') => {
                        // Switch to the data state.
                        self.switch_to(TokenizerState::Data);
                        // Emit the DOCTYPE token.
                        self.emit_current_token();
                    },
                    Some('\0') => {
                        // This is an unexpected-null-character parse error.
                        self.parse_error(HtmlParseError::UnexpectedNullCharacter);

                        // Ignore the character.
                    },
                    Some(_) => {
                        // Ignore the character.
                    },
                    None => {
                        // Emit the DOCTYPE token.
                        self.emit_current_token();

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#cdata-section-state
            TokenizerState::CDATASection => {
                // Consume the next input character:
                match self.read_next() {
                    Some(']') => {
                        // Switch to the CDATA section bracket state.
                        self.switch_to(TokenizerState::CDATASectionBracket);
                    },
                    Some(c) => {
                        // Emit the current input character as a character token.
                        self.emit(Token::Character(c));
                    },
                    None => {
                        // This is an eof-in-cdata parse error.
                        self.parse_error(HtmlParseError::EOFInCDATA);

                        // Emit an end-of-file token.
                        self.emit(Token::EOF);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#cdata-section-bracket-state
            TokenizerState::CDATASectionBracket => {
                // Consume the next input character:
                match self.read_next() {
                    Some(']') => {
                        // Switch to the CDATA section end state.
                        self.switch_to(TokenizerState::CDATASectionEnd);
                    },
                    _ => {
                        // Emit a U+005D RIGHT SQUARE BRACKET character token.
                        self.emit(Token::Character(']'));

                        // Reconsume in the CDATA section state.
                        self.reconsume_in(TokenizerState::CDATASection);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#cdata-section-end-state
            TokenizerState::CDATASectionEnd => {
                // Consume the next input character:
                match self.read_next() {
                    Some(']') => {
                        // Emit a U+005D RIGHT SQUARE BRACKET character token.
                        self.emit(Token::Character(']'));
                    },
                    Some('>') => {
                        // Switch to the data state.
                        self.switch_to(TokenizerState::Data);
                    },
                    _ => {
                        // Emit two U+005D RIGHT SQUARE BRACKET character tokens.
                        self.emit(Token::Character(']'));
                        self.emit(Token::Character(']'));

                        // Reconsume in the CDATA section state.
                        self.reconsume_in(TokenizerState::CDATASection);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#character-reference-state
            TokenizerState::CharacterReference => {
                // Set the temporary buffer to the empty string.
                // Append a U+0026 AMPERSAND (&) character to the temporary buffer.
                self.buffer.clear();
                self.buffer.push('&');

                // Consume the next input character:
                match self.read_next() {
                    Some('a'..='z' | 'A'..='Z' | '0'..='9') => {
                        // Reconsume in the named character reference state.
                        self.reconsume_in(TokenizerState::NamedCharacterReference);
                    },
                    Some('#') => {
                        // Append the current input character to the temporary buffer.
                        self.add_to_buffer('#');

                        // Switch to the numeric character reference state.
                        self.switch_to(TokenizerState::NumericCharacterReference);
                    },
                    _ => {
                        // Flush code points consumed as a character reference.
                        // NOTE: we are supposed to flush the buffer as tokens - but we just set it to "&".
                        //       Let's just emit a single '&' token i guess?
                        //       Sorry to future me if this causes any bugs :^)
                        self.emit(Token::Character('&'));

                        // Reconsume in the return state.
                        self.reconsume_in(self.return_state.unwrap());
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#named-character-reference-state
            TokenizerState::NamedCharacterReference => {
                match lookup_character_reference(self.source.remaining()) {
                    Some((matched_str, resolved_reference)) => {
                        let _ = self.source.advance_by(matched_str.len());

                        // FIXME:
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

                        // Set the temporary buffer to the empty string.
                        // Append one or two characters corresponding to
                        // the character reference name (as given by the
                        // second column of the named character references
                        // table) to the temporary buffer.
                        self.buffer.clear();
                        self.buffer.push_str(resolved_reference);

                        // Flush code points consumed as a character reference.
                        self.flush_code_points_consumed_as_character_reference();

                        // Switch to the return state.
                        self.switch_to(self.return_state.expect("No return state"));
                    },
                    None => {
                        // Flush code points consumed as a character reference.
                        self.flush_code_points_consumed_as_character_reference();

                        // Switch to the ambiguous ampersand state.
                        self.switch_to(TokenizerState::AmbiguousAmpersand);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#ambiguous-ampersand-state
            TokenizerState::AmbiguousAmpersand => {
                // Consume the next input character:
                match self.read_next() {
                    Some(c @ ('a'..='z' | 'A'..='Z' | '0'..='9')) => {
                        // If the character reference was consumed as part of an attribute,
                        if self.is_inside_attribute() {
                            // then append the current input character to the current attribute's
                            // value.
                            self.current_tag.current_attribute_value.push(c);
                        } else {
                            // Otherwise, emit the current input character as a character
                            // token.
                            self.emit(Token::Character(c));
                        }
                    },
                    Some(';') => {
                        // This is an unknown-named-character-reference parse error.
                        self.parse_error(HtmlParseError::UnknownNamedCharacterReference);

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
            TokenizerState::NumericCharacterReference => {
                // Set the character reference code to zero (0).
                self.character_reference_code = 0;

                // Consume the next input character:
                match self.read_next() {
                    Some(c @ ('X' | 'x')) => {
                        // Append the current input character to the temporary buffer.
                        self.add_to_buffer(c);

                        // Switch to the hexadecimal character reference start state.
                        self.switch_to(TokenizerState::HexadecimalCharacterReferenceStart);
                    },
                    _ => {
                        // Reconsume in the decimal character reference start state.
                        self.reconsume_in(TokenizerState::DecimalCharacterReferenceStart);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#hexadecimal-character-reference-start-state
            TokenizerState::HexadecimalCharacterReferenceStart => {
                // Consume the next input character:
                match self.read_next() {
                    Some('0'..='9' | 'a'..='f' | 'A'..='F') => {
                        // Reconsume in the hexadecimal character reference state.
                        self.reconsume_in(TokenizerState::HexadecimalCharacterReference);
                    },
                    _ => {
                        // This is an absence-of-digits-in-numeric-character-reference parse error.
                        self.parse_error(
                            HtmlParseError::AbsenceOfDigitsInNumericCharacterReference,
                        );

                        // Flush code points consumed as a character reference.
                        self.flush_code_points_consumed_as_character_reference();

                        // Reconsume in the return state.
                        self.reconsume_in(self.return_state.unwrap());
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#decimal-character-reference-start-state
            TokenizerState::DecimalCharacterReferenceStart => {
                // Consume the next input character:
                match self.read_next() {
                    Some('0'..='9') => {
                        // Reconsume in the decimal character reference state.
                        self.reconsume_in(TokenizerState::DecimalCharacterReference);
                    },
                    _ => {
                        // This is an absence-of-digits-in-numeric-character-reference parse
                        // error.
                        self.parse_error(
                            HtmlParseError::AbsenceOfDigitsInNumericCharacterReference,
                        );

                        // Flush code points consumed as a character reference.
                        self.flush_code_points_consumed_as_character_reference();

                        // Reconsume in the return state.
                        self.reconsume_in(self.return_state.unwrap());
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#hexadecimal-character-reference-state
            TokenizerState::HexadecimalCharacterReference => {
                // Consume the next input character:
                match self.read_next() {
                    Some(c @ ('0'..='9' | 'a'..='f' | 'A'..='F')) => {
                        // Multiply the character reference code by 16.
                        self.character_reference_code =
                            self.character_reference_code.saturating_mul(16);

                        // Add a numeric version of the current input character to the character reference code.
                        self.character_reference_code =
                            self.character_reference_code.saturating_add(
                                c.to_digit(16)
                                    .expect("characters 0-9, a-f, A-F are valid hex digits"),
                            );
                    },
                    Some(';') => {
                        // Switch to the numeric character reference end state.
                        self.switch_to(TokenizerState::NumericCharacterReferenceEnd);
                    },
                    _ => {
                        // This is a missing-semicolon-after-character-reference parse error.
                        self.parse_error(HtmlParseError::MissingSemicolonAfterCharacterReference);

                        // Reconsume in the numeric character reference end state.
                        self.reconsume_in(TokenizerState::NumericCharacterReferenceEnd);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#decimal-character-reference-state
            TokenizerState::DecimalCharacterReference => {
                // Consume the next input character:
                match self.read_next() {
                    Some(c @ '0'..='9') => {
                        // Multiply the character reference code by 10.
                        self.character_reference_code =
                            self.character_reference_code.saturating_mul(10);

                        // Add a numeric version of
                        // the current input character (subtract 0x0030 from the character's code
                        // point) to the character reference code.
                        self.character_reference_code =
                            self.character_reference_code.saturating_add(
                                c.to_digit(10)
                                    .expect("characters 0-9 are valid decimal digits"),
                            );
                    },
                    Some(';') => {
                        // Switch to the numeric character reference end state.
                        self.switch_to(TokenizerState::NumericCharacterReferenceEnd);
                    },
                    _ => {
                        // This is a missing-semicolon-after-character-reference parse error.
                        self.parse_error(HtmlParseError::MissingSemicolonAfterCharacterReference);

                        // Reconsume in the numeric character reference end state.
                        self.reconsume_in(TokenizerState::NumericCharacterReferenceEnd);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#numeric-character-reference-end-state
            TokenizerState::NumericCharacterReferenceEnd => {
                // Check the character reference code:
                match self.character_reference_code {
                    0x00 => {
                        // This is a null-character-reference parse error.
                        self.parse_error(HtmlParseError::NullCharacterReference);

                        // Set the character reference code to 0xFFFD.
                        self.character_reference_code = 0xFFFD;
                    },
                    0x110000.. => {
                        // This is a character-reference-outside-unicode-range parse error.
                        self.parse_error(HtmlParseError::CharacterReferenceOutsideOfUnicodeRange);

                        // Set the character reference code to 0xFFFD.
                        self.character_reference_code = 0xFFFD;
                    },
                    0xD800..=0xDFFF => {
                        // This is a surrogate-character-reference parse error.
                        self.parse_error(HtmlParseError::SurrogateCharacterReference);

                        // Set the character reference code to 0xFFFD.
                        self.character_reference_code = 0xFFFD;
                    },
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
                        self.parse_error(HtmlParseError::NoncharacterCharacterReference);
                    },
                    c @ (0x0D | 0xC0 | 0x007F..=0x009F) => {
                        if c != TAB as u32
                            || c != LINE_FEED as u32
                            || c != FORM_FEED as u32
                            || c != SPACE as u32
                        {
                            // This is a control character reference parse error
                            self.parse_error(HtmlParseError::ControlCharacterReference);
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
                // Set the temporary buffer to the empty string.
                self.buffer.clear();

                // Append a code point equal to the character reference code to the temporary buffer.
                self.buffer
                    .push(char::from_u32(self.character_reference_code).unwrap());

                // Flush code points consumed as a character reference.
                self.flush_code_points_consumed_as_character_reference();

                // Switch to the return state.
                self.switch_to(self.return_state.unwrap());
            },
        }
    }
}

impl<P: ParseErrorHandler> Iterator for Tokenizer<P> {
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
