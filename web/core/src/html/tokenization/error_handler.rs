#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HtmlParseError {
    /// No dedicated error code
    Generic,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-abrupt-closing-of-empty-comment>
    AbruptClosingOfEmptyComment,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-abrupt-doctype-public-identifier>
    AbruptDoctypePublicIdentifier,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-abrupt-doctype-system-identifier>
    AbruptDoctypeSystemIdentifier,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-absence-of-digits-in-numeric-character-reference>
    AbsenceOfDigitsInNumericCharacterReference,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-cdata-in-html-content>
    CDATAInHtmlContent,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-character-reference-outside-unicode-range>
    CharacterReferenceOutsideOfUnicodeRange,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-control-character-in-input-stream>
    ControlCharacterInInputStream,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-control-character-reference>
    ControlCharacterReference,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-end-tag-with-attributes>
    EndTagWithAttributes,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-duplicate-attribute>
    DuplicateAttribute,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-end-tag-with-trailing-solidus>
    EndTagWithTrailingSolidus,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-eof-before-tag-name>
    EOFBeforeTagName,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-eof-in-cdata>
    EOFInCDATA,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-eof-in-comment>
    EOFInComment,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-eof-in-doctype>
    EOFInDoctype,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-eof-in-script-html-comment-like-text>
    EOFInScriptHtmlCommentLikeText,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-eof-in-tag>
    EOFInTag,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-incorrectly-closed-comment>
    IncorrectlyClosedComment,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-incorrectly-opened-comment>
    IncorrectlyOpenedComment,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-invalid-character-sequence-after-doctype-name>
    InvalidCharacterSequenceAfterDoctypeName,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-invalid-first-character-of-tag-name>
    InvalidFirstCharacterOfTagName,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-missing-attribute-value>
    MissingAttributeValue,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-missing-doctype-name>
    MissingDoctypeName,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-missing-doctype-public-identifier>
    MissingDoctypePublicIdentifier,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-missing-doctype-system-identifier>
    MissingDoctypeSystemIdentifier,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-missing-end-tag-name>
    MissingEndTagName,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-missing-quote-before-doctype-public-identifier>
    MissingQuoteBeforeDoctypePublicIdentifier,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-missing-quote-before-doctype-system-identifier>
    MissingQuoteBeforeDoctypeSystemIdentifier,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-missing-semicolon-after-character-reference>
    MissingSemicolonAfterCharacterReference,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-missing-whitespace-after-doctype-public-keyword>
    MissingWhitespaceAfterDoctypePublicKeyword,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-missing-whitespace-after-doctype-system-keyword>
    MissingWhitespaceAfterDoctypeSystemKeyword,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-missing-whitespace-before-doctype-name>
    MissingWhitespaceBeforeDoctypeName,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-missing-whitespace-between-attributes>
    MissingWhitespaceBetweenAttributes,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-missing-whitespace-between-doctype-public-and-system-identifiers>
    MissingWhitespaceBetweenDoctypePublicAndSystemIdentifier,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-nested-comment>
    NestedComment,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-noncharacter-character-reference>
    NoncharacterCharacterReference,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-noncharacter-in-input-stream>
    NoncharacterInInputStream,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-non-void-html-element-start-tag-with-trailing-solidus>
    NonVoidHtmlElementStartTagWithTrailingSolidus,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-null-character-reference>
    NullCharacterReference,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-surrogate-character-reference>
    SurrogateCharacterReference,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-surrogate-in-input-stream>
    SurrogateInInputStream,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-unexpected-character-after-doctype-system-identifier>
    UnexpectedCharacterAfterDoctypeSystemIdentifier,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-unexpected-character-in-attribute-name>
    UnexpectedCharacterInAttributeName,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-unexpected-character-in-unquoted-attribute-value>
    UnexpectedCharacterInUnquotedAttributeValue,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-unexpected-equals-sign-before-attribute-name>
    UnexpectedEqualsSignBeforeAttributeName,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-unexpected-null-character>
    UnexpectedNullCharacter,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-unexpected-question-mark-instead-of-tag-name>
    UnexpectedQuestionMarkInsteadOfTagName,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-unexpected-solidus-in-tag>
    UnexpectedSolidusInTag,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-error-unknown-named-character-reference>
    UnknownNamedCharacterReference,
}

pub trait ParseErrorHandler {
    fn handle(error: HtmlParseError);
}

pub struct IgnoreParseErrors;

impl ParseErrorHandler for IgnoreParseErrors {
    fn handle(error: HtmlParseError) {
        _ = error;
    }
}
