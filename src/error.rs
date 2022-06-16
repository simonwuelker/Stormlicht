pub enum TokenizerError {
    /// This error occurs if the parser encounters an empty comment that is abruptly closed by a U+003E (>) code point (i.e., <!--> or <!--->). The parser behaves as if the comment is closed correctly.
    /// abrupt-doctype-public-identifier 	
    AbruptClosingOfEmptyComment,

    /// This error occurs if the parser encounters a U+003E (>) code point in the DOCTYPE public identifier (e.g., <!DOCTYPE html PUBLIC "foo>). In such a case, if the DOCTYPE is correctly placed as a document preamble, the parser sets the Document to quirks mode.
    AbruptDoctypePublicIdentifier,

    /// This error occurs if the parser encounters a U+003E (>) code point in the DOCTYPE system identifier (e.g., <!DOCTYPE html PUBLIC "-//W3C//DTD HTML 4.01//EN" "foo>). In such a case, if the DOCTYPE is correctly placed as a document preamble, the parser sets the Document to quirks mode.
    AbruptDoctypeSystemIdentifier,

    /// This error occurs if the parser encounters a numeric character reference that doesn't contain any digits (e.g., &#qux;). In this case the parser doesn't resolve the character reference.
    AbsenceOfDigitsInNumericCharacterReference,

    /// This error occurs if the parser encounters a CDATA section outside of foreign content (SVG or MathML). The parser treats such CDATA sections (including leading "[CDATA[" and trailing "]]" strings) as comments.
    CdataInHtmlContent,

    /// This error occurs if the parser encounters a numeric character reference that references a code point that is greater than the valid Unicode range. The parser resolves such a character reference to a U+FFFD REPLACEMENT CHARACTER.
    CharacterReferenceOutsideUnicodeRane,

    /// This error occurs if the input stream contains a control code point that is not ASCII whitespace or U+0000 NULL. Such code points are parsed as-is and usually, where parsing rules don't apply any additional restrictions, make their way into the DOM.
    ControlCharacterInInputStream,

    /// This error occurs if the parser encounters a numeric character reference that references a control code point that is not ASCII whitespace or is a U+000D CARRIAGE RETURN. The parser resolves such character references as-is except C1 control references that are replaced according to the numeric character reference end state.
    ControlCharacterReference,

    /// This error occurs if the parser encounters an end tag with attributes. Attributes in end tags are ignored and do not make their way into the DOM.
    EndTagWithAttributes,

    /// This error occurs if the parser encounters an attribute in a tag that already has an attribute with the same name. The parser ignores all such duplicate occurrences of the attribute.
    DuplicateAttribute,

    /// This error occurs if the parser encounters an end tag that has a U+002F (/) code point right before the closing U+003E (>) code point (e.g., </div/>). Such a tag is treated as a regular end tag.
    EndTagWithTrailingSolidus,

    /// This error occurs if the parser encounters the end of the input stream where a tag name is expected. In this case the parser treats the beginning of a start tag (i.e., <) or an end tag (i.e., </) as text content.
    EofBeforeTagName,

    /// This error occurs if the parser encounters the end of the input stream in a CDATA section. The parser treats such CDATA sections as if they are closed immediately before the end of the input stream.
    EofInCdata,

    /// This error occurs if the parser encounters the end of the input stream in a comment. The parser treats such comments as if they are closed immediately before the end of the input stream.
    EofInComment,

    /// This error occurs if the parser encounters the end of the input stream in a DOCTYPE. In such a case, if the DOCTYPE is correctly placed as a document preamble, the parser sets the Document to quirks mode.
    EofInDoctype,

    /// This error occurs if the parser encounters the end of the input stream in text that resembles an HTML comment inside script element content (e.g., <script><!-- foo).
    /// Syntactic structures that resemble HTML comments in script elements are parsed as text content. They can be a part of a scripting language-specific syntactic structure or be treated as an HTML-like comment, if the scripting language supports them (e.g., parsing rules for HTML-like comments can be found in Annex B of the JavaScript specification). The common reason for this error is a violation of the restrictions for contents of script elements.
    EofInScriptHtmlCommentLikeText,

    /// This error occurs if the parser encounters the end of the input stream in a start tag or an end tag (e.g., <div id=). Such a tag is ignored.
    EofInTag,

    /// This error occurs if the parser encounters a comment that is closed by the "--!>" code point sequence. The parser treats such comments as if they are correctly closed by the "-->" code point sequence.
    IncorrectlyClosedComment,

    /// This error occurs if the parser encounters the "<!" code point sequence that is not immediately followed by two U+002D (-) code points and that is not the start of a DOCTYPE or a CDATA section. All content that follows the "<!" code point sequence up to a U+003E (>) code point (if present) or to the end of the input stream is treated as a comment.
    /// One possible cause of this error is using an XML markup declaration (e.g., <!ELEMENT br EMPTY>) in HTML.
    IncorrectlyOpenedComment,

    /// This error occurs if the parser encounters any code point sequence other than "PUBLIC" and "SYSTEM" keywords after a DOCTYPE name. In such a case, the parser ignores any following public or system identifiers, and if the DOCTYPE is correctly placed as a document preamble, and if the parser cannot change the mode flag is false, sets the Document to quirks mode.
    InvalidCharacterSequenceAfterDoctypeName,

    /// This error occurs if the parser encounters a code point that is not an ASCII alpha where first code point of a start tag name or an end tag name is expected. If a start tag was expected such code point and a preceding U+003C (<) is treated as text content, and all content that follows is treated as markup. Whereas, if an end tag was expected, such code point and all content that follows up to a U+003E (>) code point (if present) or to the end of the input stream is treated as a comment.

    /// For example, consider the following markup:
    /// 
    /// ```ignore
    /// <42></42>
    /// ```
    /// 
    /// This will be parsed into:
    /// 
    /// ```ignore
    ///     html
    ///         head
    ///         body
    ///             #text: <42>
    ///             #comment: 42
    /// ```
    /// 
    /// While the first code point of a tag name is limited to an ASCII alpha, a wide range of code points (including ASCII digits) is allowed in subsequent positions.
    InvalidFirstCharacterOfTagName,


    /// This error occurs if the parser encounters a U+003E (>) code point where an attribute value is expected (e.g., <div id=>). The parser treats the attribute as having an empty value.
    MissingAttributeValue,

    /// This error occurs if the parser encounters a DOCTYPE that is missing a name (e.g., <!DOCTYPE>). In such a case, if the DOCTYPE is correctly placed as a document preamble, the parser sets the Document to quirks mode.
    MissingDoctypeName,

    /// This error occurs if the parser encounters a U+003E (>) code point where start of the DOCTYPE public identifier is expected (e.g., <!DOCTYPE html PUBLIC >). In such a case, if the DOCTYPE is correctly placed as a document preamble, the parser sets the Document to quirks mode.
    MissingDoctypePublicIdentifier,

    /// This error occurs if the parser encounters a U+003E (>) code point where start of the DOCTYPE system identifier is expected (e.g., <!DOCTYPE html SYSTEM >). In such a case, if the DOCTYPE is correctly placed as a document preamble, the parser sets the Document to quirks mode.
    MissingDoctypeSystemIdentifier,

    /// This error occurs if the parser encounters a U+003E (>) code point where an end tag name is expected, i.e., </>. The parser ignores the whole "</>" code point sequence.
    MissingEndTagName,

    /// This error occurs if the parser encounters the DOCTYPE public identifier that is not preceded by a quote (e.g., <!DOCTYPE html PUBLIC -//W3C//DTD HTML 4.01//EN">). In such a case, the parser ignores the public identifier, and if the DOCTYPE is correctly placed as a document preamble, sets the Document to quirks mode.
    MissingQuoteBeforeDoctypePublicIdentifier,

    /// This error occurs if the parser encounters the DOCTYPE system identifier that is not preceded by a quote (e.g., <!DOCTYPE html SYSTEM http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd">). In such a case, the parser ignores the system identifier, and if the DOCTYPE is correctly placed as a document preamble, sets the Document to quirks mode.
    MissingQuoteBeforeDoctypeSystemIdentifier,

    /// This error occurs if the parser encounters a character reference that is not terminated by a U+003B (;) code point. Usually the parser behaves as if character reference is terminated by the U+003B (;) code point; however, there are some ambiguous cases in which the parser includes subsequent code points in the character reference.
    /// For example, &not;in will be parsed as "¬in" whereas &notin will be parsed as "∉".
    MissingSemicolonAfterCharacterReference,

    /// This error occurs if the parser encounters a DOCTYPE whose "PUBLIC" keyword and public identifier are not separated by ASCII whitespace. In this case the parser behaves as if ASCII whitespace is present.
    MissingWhitespaceAfterDoctypePublicKeyword,

    /// This error occurs if the parser encounters a DOCTYPE whose "SYSTEM" keyword and system identifier are not separated by ASCII whitespace. In this case the parser behaves as if ASCII whitespace is present.
    MissingWhitespaceAfterDoctypeSystemKeyword,

    /// This error occurs if the parser encounters a DOCTYPE whose "DOCTYPE" keyword and name are not separated by ASCII whitespace. In this case the parser behaves as if ASCII whitespace is present.
    MissingWhitespaceBeforeDoctypeName,

    /// This error occurs if the parser encounters attributes that are not separated by ASCII whitespace (e.g., <div id="foo"class="bar">). In this case the parser behaves as if ASCII whitespace is present.
    MissingWhitespaceBetweenAttributes,

    /// This error occurs if the parser encounters a DOCTYPE whose public and system identifiers are not separated by ASCII whitespace. In this case the parser behaves as if ASCII whitespace is present.
    MissingWhitespaceBetweenDoctypePublicAndSystemIdentifiers,

    /// This error occurs if the parser encounters a nested comment (e.g., <!-- <!-- nested --> -->). Such a comment will be closed by the first occurring "-->" code point sequence and everything that follows will be treated as markup.
    NestedComment,

    /// This error occurs if the parser encounters a numeric character reference that references a noncharacter. The parser resolves such character references as-is.
    NoncharacterCharacterReference,

    /// This error occurs if the input stream contains a noncharacter. Such code points are parsed as-is and usually, where parsing rules don't apply any additional restrictions, make their way into the DOM.
    NoncharacterInInputStream,

    /// This error occurs if the parser encounters a start tag for an element that is not in the list of void elements or is not a part of foreign content (i.e., not an SVG or MathML element) that has a U+002F (/) code point right before the closing U+003E (>) code point. The parser behaves as if the U+002F (/) is not present.
    /// For example, consider the following markup:
    /// 
    /// ```ignore
    /// <div/><span></span><span></span>
    /// ```
    /// 
    /// This will be parsed into:
    /// ```ignore
    /// 
    ///     html
    ///         head
    ///         body
    ///             div
    ///                 span
    ///                 span
    /// ```
    /// 
    /// The trailing U+002F (/) in a start tag name can be used only in foreign content to specify self-closing tags. (Self-closing tags don't exist in HTML.) It is also allowed for void elements, but doesn't have any effect in this case.
    NonVoidHtmlElementStartTagWithTrailingSolidus,


    /// This error occurs if the parser encounters a numeric character reference that references a U+0000 NULL code point. The parser resolves such character references to a U+FFFD REPLACEMENT CHARACTER.
    NullCharacterReference,

    /// This error occurs if the parser encounters a numeric character reference that references a surrogate. The parser resolves such character references to a U+FFFD REPLACEMENT CHARACTER.
    SurrogateCharacterReference,

    /// This error occurs if the input stream contains a surrogate. Such code points are parsed as-is and usually, where parsing rules don't apply any additional restrictions, make their way into the DOM.
    /// Surrogates can only find their way into the input stream via script APIs such as document.write().
    SurrogateInInputStream,


    /// This error occurs if the parser encounters any code points other than ASCII whitespace or closing U+003E (>) after the DOCTYPE system identifier. The parser ignores these code points.
    UnexpectedCharacterAfterDoctypeSystemIdentifier,

    /// This error occurs if the parser encounters a U+0022 ("), U+0027 ('), or U+003C (<) code point in an attribute name. The parser includes such code points in the attribute name.
    /// Code points that trigger this error are usually a part of another syntactic construct and can be a sign of a typo around the attribute name.
    /// For example, consider the following markup:
    /// ```ignore
    /// <div foo<div>
    /// ```
    /// Due to a forgotten U+003E (>) code point after foo the parser treats this markup as a single div element with a "foo<div" attribute.
    /// 
    /// As another example of this error, consider the following markup:
    /// ```ignore
    /// <div id'bar'>
    /// ```
    /// Due to a forgotten U+003D (=) code point between an attribute name and value the parser treats this markup as a div element with the attribute "id'bar'" that has an empty value.
    UnexpectedCharacterInAttributeName,


    /// This error occurs if the parser encounters a U+0022 ("), U+0027 ('), U+003C (<), U+003D (=), or U+0060 (`) code point in an unquoted attribute value. The parser includes such code points in the attribute value.
    /// Code points that trigger this error are usually a part of another syntactic construct and can be a sign of a typo around the attribute value.
    /// U+0060 (`) is in the list of code points that trigger this error because certain legacy user agents treat it as a quote.
    /// For example, consider the following markup:
    /// ```ignore
    /// <div foo=b'ar'>
    /// ```
    /// Due to a misplaced U+0027 (') code point the parser sets the value of the "foo" attribute to "b'ar'".
    /// unexpected-equals-sign-before-attribute-name 	
    /// This error occurs if the parser encounters a U+003D (=) code point before an attribute name. In this case the parser treats U+003D (=) as the first code point of the attribute name.
    /// The common reason for this error is a forgotten attribute name.
    /// For example, consider the following markup:
    /// ```ignore
    /// <div foo="bar" ="baz">
    /// ```
    /// Due to a forgotten attribute name the parser treats this markup as a div element with two attributes: a "foo" attribute with a "bar" value and a "="baz"" attribute with an empty value.
    UnexpectedCharacterInUnquotedAttributeValue,


    /// This error occurs if the parser encounters a U+0000 NULL code point in the input stream in certain positions. In general, such code points are either ignored or, for security reasons, replaced with a U+FFFD REPLACEMENT CHARACTER.
    UnexpectedNullCharacter,

    /// This error occurs if the parser encounters a U+003F (?) code point where first code point of a start tag name is expected. The U+003F (?) and all content that follows up to a U+003E (>) code point (if present) or to the end of the input stream is treated as a comment.
    /// For example, consider the following markup:
    /// ```ignore
    /// <?xml-stylesheet type="text/css" href="style.css"?>
    /// ```
    /// This will be parsed into:
    /// ```ignore
    ///     #comment: ?xml-stylesheet type="text/css" href="style.css"?
    ///     html
    ///         head
    ///         body
    /// ```
    /// The common reason for this error is an XML processing instruction (e.g., <?xml-stylesheet type="text/css" href="style.css"?>) or an XML declaration (e.g., <?xml version="1.0" encoding="UTF-8"?>) being used in HTML.
    UnexpectedQuestionMarkInsteadOfTagName,

    /// This error occurs if the parser encounters a U+002F (/) code point that is not a part of a quoted attribute value and not immediately followed by a U+003E (>) code point in a tag (e.g., <div / id="foo">). In this case the parser behaves as if it encountered ASCII whitespace.
    UnexpectedSolidusInTag,

    /// This error occurs if the parser encounters an ambiguous ampersand. In this case the parser doesn't resolve the character reference.
    UnknownNamedCharacterReference,
}
