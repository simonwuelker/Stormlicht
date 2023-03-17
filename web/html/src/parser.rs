//! Implements the [Tree Construction Stage](https://html.spec.whatwg.org/multipage/parsing.html#tree-construction)

use crate::{
    infra::Namespace,
    tokenizer::{TagData, Token, Tokenizer, TokenizerState},
}

use dom::{
    dom_objects::{Comment, Document, DocumentType, Node, Text},
    DOMPtr, DOMType,
};

// TODO
// The scopes are not implemented yet.
// The only reason they contain the html type is because otherwise, is_element_in_scope
// doesn't terminate.
const DEFAULT_SCOPE: [DOMType; 0] = [];
const BUTTON_SCOPE: [DOMType; 0] = [];

const TAB: char = '\u{0009}';
const LINE_FEED: char = '\u{000A}';
const FORM_FEED: char = '\u{000C}';
const WHITESPACE: char = '\u{0020}';

// const HEADINGS: [DOMType; 6] = [
//     DOMType::H1,
//     DOMType::H2,
//     DOMType::H3,
//     DOMType::H4,
//     DOMType::H5,
//     DOMType::H6,
// ];

#[derive(Clone, Copy, Debug)]
enum GenericParsingAlgorithm {
    RCDATA,
    RawText,
}

#[derive(Debug, Clone, Copy)]
/// <https://html.spec.whatwg.org/multipage/parsing.html#parse-state>
pub enum InsertionMode {
    Initial,
    BeforeHtml,
    BeforeHead,
    InHead,
    InHeadNoscript,
    AfterHead,
    InBody,
    Text,
    InTable,
    InTableText,
    InCaption,
    InColumnGroup,
    InTableBody,
    InRow,
    InCell,
    InSelect,
    InSelectInTable,
    InTemplate,
    AfterBody,
    InFrameset,
    AfterFrameset,
    AfterAfterBody,
    AfterAfterFrameset,
}

pub struct Parser<'source> {
    tokenizer: Tokenizer<'source>,
    /// When the insertion mode is switched to "text" or "in table text", the original insertion
    /// mode is also set. This is the insertion mode to which the tree construction stage will
    /// return.
    original_insertion_mode: Option<InsertionMode>,
    insertion_mode: InsertionMode,
    open_elements: Vec<DOMPtr<Node>>,
    head: Option<DOMPtr<Node>>,
    scripting: bool,
    done: bool,
}

impl<'source> Parser<'source> {
    pub fn new(source: &'source str) -> Self {
        let document = DOMPtr::new(Document::default())
            .into_type::<Node>();

        Self {
            tokenizer: Tokenizer::new(source),
            original_insertion_mode: None,
            insertion_mode: InsertionMode::Initial,
            open_elements: vec![document],
            head: None,
            scripting: false,
            done: false,
        }
    }

    pub fn parse(mut self) -> DOMPtr<Node> {
        while let Some(token) = self.tokenizer.next() {
            self.consume(token);

            if self.done {
                break;
            }
        }

        DOMPtr::clone(&self.open_elements[0])
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#current-node>
    fn current_node(&self) -> DOMPtr<Node> {
        // The current node is the bottommost node in this stack of open elements.
        DOMPtr::clone(self.open_elements.first().unwrap())
    }

    /// Convenience method to get the current [Document] node
    fn document(&self) -> DOMPtr<Document> {
        DOMPtr::clone(self.open_elements.first().unwrap()).into_type::<Document>()
    }

    fn consume(&mut self, token: Token) {
        self.consume_in_mode(self.insertion_mode, token);
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#insert-a-character>
    pub fn insert_character(&self, c: char) {
        // FIXME Let data be the characters passed to the algorithm, or, if no characters were explicitly specified, the character of the character token being processed.

        // Let the adjusted insertion location be the appropriate place for inserting a node.
        let adjusted_insert_location = self.appropriate_place_for_inserting_node();

        // If the adjusted insertion location is in a Document node
        if adjusted_insert_location.is_a::<Document>() {
            // then return.
            return;
        }

        // If there is a Text node immediately before the adjusted insertion location
        if let Some(last_child) = adjusted_insert_location.borrow().last_child() {
            if last_child.is_a::<Text>() {
                let text = last_child.into_type::<Text>();
                // then append data to that Text node's data.
                text.borrow_mut().content_mut().push(c);
            }
        }

        // Otherwise, create a new Text node whose data is data and whose node document is the same as
        // that of the element in which the adjusted insertion location finds itself
        let document = adjusted_insert_location
            .borrow()
            .owning_document()
            .unwrap()
            .downgrade();

        let mut new_text = Text::default();
        new_text.content_mut().push(c);
        new_text.set_owning_document(document);

        // and insert the newly created node at the adjusted insertion location.
        let new_node = DOMPtr::new(new_text).into_type();
        Node::append_child(adjusted_insert_location, new_node)
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#appropriate-place-for-inserting-a-node>
    fn appropriate_place_for_inserting_node(&self) -> DOMPtr<Node> {
        self.appropriate_place_for_inserting_node_with_override(None)
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#appropriate-place-for-inserting-a-node>
    fn appropriate_place_for_inserting_node_with_override(
        &self,
        override_target: Option<DOMPtr<Node>>,
    ) -> DOMPtr<Node> {
        // If there was an override target specified, then let target be the override target.
        // Otherwise, let target be the current node.
        override_target.unwrap_or_else(|| self.current_node())

        // TODO: the specificaiton  talks about foster parenting here, which we don't support

        // Let adjusted insertion location be inside target, after its last child (if any).

        // TODO:
        // If the adjusted insertion location is inside a template element, let it instead be
        // inside the template element's template contents, after its last child (if any).

        // Return the adjusted insertion location.
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#insert-a-comment>
    fn insert_comment_at(&mut self, data: String, position: Option<DOMPtr<Node>>) {
        // Let data be the data given in the comment token being processed.

        // If position was specified, then let the adjusted insertion location be position.
        // Otherwise, let adjusted insertion location be the appropriate place for inserting a node.
        let adjusted_insert_location =
            position.unwrap_or_else(|| self.appropriate_place_for_inserting_node());

        // Create a Comment node whose data attribute is set to data and whose node document is the same
        // as that of the node in which the adjusted insertion location finds itself.
        let document = adjusted_insert_location
            .borrow()
            .owning_document()
            .unwrap()
            .downgrade();

        let mut new_comment = Comment::default();
        new_comment.content_mut().push_str(&data);
        new_comment.set_owning_document(document);

        // Insert the newly created node at the adjusted insertion location.
        let new_node = DOMPtr::new(new_comment).into_type();
        Node::append_child(adjusted_insert_location, new_node);
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#insert-a-comment>
    fn insert_comment(&mut self, data: String) {
        self.insert_comment_at(data, None);
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parsing-elements-that-contain-only-text>
    fn generic_parsing_algorithm(&mut self, tagdata: TagData, algorithm: GenericParsingAlgorithm) {
        // Insert an HTML element for the token.
        self.insert_html_element_for_token(&tagdata);

        // If the algorithm that was invoked is the generic raw text element parsing algorithm,
        // switch the tokenizer to the RAWTEXT state;
        // otherwise the algorithm invoked was the generic RCDATA element parsing algorithm,
        // switch the tokenizer to the RCDATA state.
        match algorithm {
            GenericParsingAlgorithm::RawText => {
                self.tokenizer.switch_to(TokenizerState::RAWTEXTState)
            },
            GenericParsingAlgorithm::RCDATA => {
                self.tokenizer.switch_to(TokenizerState::RCDATAState)
            },
        }

        // Let the original insertion mode be the current insertion mode.
        self.original_insertion_mode = Some(self.insertion_mode);

        // Then, switch the insertion mode to "text".
        self.insertion_mode = InsertionMode::Text;
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#has-an-element-in-scope>
    fn is_element_in_scope(&self, _target_node_type: &DOMType, _scope: &[DOMType]) -> bool {
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#close-a-p-element>
    fn close_p_element(&mut self) {
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#closing-elements-that-have-implied-end-tags>
    fn generate_implied_end_tags(&mut self) {
        self.generate_implied_end_tags_excluding(None);
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#closing-elements-that-have-implied-end-tags>
    fn generate_implied_end_tags_excluding(&mut self, _exclude: Option<DOMType>) {
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#create-an-element-for-the-token>
    fn create_html_element_for_token(&self, tagdata: &TagData, namespace: Namespace, intended_parent: DOMPtr<Node>) -> DOMPtr<Node> {
        // FIXME: If the active speculative HTML parser is not null, then return the result of creating a speculative mock element 
        // given given namespace, the tag name of the given token, and the attributes of the given token.

        // FIXME: Otherwise, optionally create a speculative mock element given given namespace, the tag name of the given token, and the attributes of the given token.

        // Let document be intended parent's node document.
        let document = intended_parent.borrow().owning_document();

        // Let local name be the tag name of the token.
        let local_name = tagdata.name;

        // Let is be the value of the "is" attribute in the given token, if such an attribute exists, or null otherwise.
        let is = tagdata.lookup_attribute("is");

        // Let definition be the result of looking up a custom element definition given document, given namespace, local name, and is.
        let _definition = self.lookup_custom_element_definition(namespace, &local_name, is);

        // FIXME: If definition is non-null and the parser was not created as part of the HTML fragment parsing algorithm, then let will execute script be true. Otherwise, let it be false.

        // FIXME: If will execute script is true, then:
        //      Increment document's throw-on-dynamic-markup-insertion counter.
        //      If the JavaScript execution context stack is empty, then perform a microtask checkpoint.
        //      Push a new element queue onto document's relevant agent's custom element reactions stack.

        // Let element be the result of creating an element given document, localName, given namespace, null, and is. 
        // FIXME: If will execute script is true, set the synchronous custom elements flag; otherwise, leave it unset.
        let element = self.create_element(document, local_name, namespace, None, is);

        // FIXME: Append each attribute in the given token to element.

        // FIXME: If will execute script is true, then:
        //      Let queue be the result of popping from document's relevant agent's custom element reactions stack. (This will be the same element queue as was pushed above.)
        //      Invoke custom element reactions in queue.
        //      Decrement document's throw-on-dynamic-markup-insertion counter.

        // FIXME: If element has an xmlns attribute in the XMLNS namespace whose value is not exactly the same as the element's namespace, that is a parse error. 
        // Similarly, if element has an xmlns:xlink attribute in the XMLNS namespace whose value is not the XLink Namespace, that is a parse error.

        // FIXME: If element is a resettable element, invoke its reset algorithm. (This initializes the element's value and checkedness based on the element's attributes.)

        // FIXME: If element is a form-associated element and not a form-associated custom element, the form element pointer is not null, 
        // there is no template element on the stack of open elements, element is either not listed or doesn't have a form attribute, 
        // and the intended parent is in the same tree as the element pointed to by the form element pointer, then associate element 
        // with the form element pointed to by the form element pointer and set element's parser inserted flag.

        // Return element.
        element
    }

    /// <https://html.spec.whatwg.org/multipage/custom-elements.html#look-up-a-custom-element-definition>
    fn lookup_custom_element_definition(&self, namespace: Namespace, local_name: &str, is: Option<&str>) -> Option<()> {
        // If namespace is not the HTML namespace
        if namespace != Namespace::HTML {
            // return null.
            return None;
        }

        // FIXME: If document's browsing context is null, return null.

        // FIXME: Let registry be document's relevant global object's CustomElementRegistry object.

        // FIXME: If there is custom element definition in registry with name and local name both equal to localName, return that custom element definition.

        // FIXME: If there is a custom element definition in registry with name equal to is and local name equal to localName, return that custom element definition.

        // Return null.
        None
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#insert-an-html-element>
    fn insert_html_element_for_token(&mut self, _tagdata: &TagData) {
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inhtml>
    fn consume_in_mode(&mut self, mode: InsertionMode, token: Token) {
        match mode {
            // https://html.spec.whatwg.org/multipage/parsing.html#the-initial-insertion-mode
            InsertionMode::Initial => {
                match token {
                    Token::Character(TAB | LINE_FEED | FORM_FEED | WHITESPACE) => {
                        // Ignore the token.
                    },
                    Token::Comment(data) => {
                        // Insert a comment as the last child of the Document object.
                        self.insert_comment(data);
                    },
                    Token::DOCTYPE(doctype_token) => {
                        // If the DOCTYPE token's name is not "html", or the token's public identifier is not missing,
                        // or the token's system identifier is neither missing nor "about:legacy-compat", then there is a parse error.

                        // Append a DocumentType node to the Document node, with its name set to the name given in the DOCTYPE token,
                        // or the empty string if the name was missing; its public ID set to the public identifier given in the DOCTYPE token,
                        // or the empty string if the public identifier was missing; and its system ID set to the system identifier given in
                        // the DOCTYPE token, or the empty string if the system identifier was missing.
                        let mut doctype_node = DocumentType::default();
                        *doctype_node.name_mut() = doctype_token.name.unwrap_or_default();
                        *doctype_node.public_id_mut() =
                            doctype_token.public_ident.unwrap_or_default();
                        *doctype_node.system_id_mut() =
                            doctype_token.system_ident.unwrap_or_default();

                        // FIXME: Then, if the document is not an iframe srcdoc document, and the parser cannot change the mode flag is false,
                        // and the DOCTYPE token matches one of the conditions in the following list, then set the Document to quirks mode:
                        let new_node = DOMPtr::new(doctype_node).into_type();
                        Node::append_child(self.document().into_type(), new_node);
                    },
                    _ => {
                        // FIXME: If the document is not an iframe srcdoc document, then this is a parse error;
                        // if the parser cannot change the mode flag is false, set the Document to quirks mode.

                        // In any case, switch the insertion mode to "before html", then reprocess the token.
                        self.insertion_mode = InsertionMode::BeforeHtml;
                        self.consume(token);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#the-before-html-insertion-mode
            InsertionMode::BeforeHtml => {
                match token {
                    Token::Character(TAB | LINE_FEED | FORM_FEED | WHITESPACE) => {
                        // Ignore the token.
                    },
                    Token::Comment(data) => {
                        // Insert a comment as the last child of the Document object.
                        self.insert_comment(data)
                    },
                    Token::DOCTYPE(_) => {}, // parse error, ignore token
                    Token::Tag(ref tagdata)
                        if !tagdata.opening
                            && tagdata.name != "head"
                            && tagdata.name != "body"
                            && tagdata.name != "html"
                            && tagdata.name != "br" =>
                    {
                        // Parse error. Ignore the token.
                    },
                    Token::Tag(ref tagdata) if tagdata.opening && tagdata.name == "html" => {
                        // Create an element for the token in the HTML namespace, with the Document as the intended parent.
                        let element = self.create_html_element_for_token(tagdata, self.document().into_type::<Node>());
                        todo!()
                        // Append it to the Document object.

                        // Put this element in the stack of open elements.
                        self.open_elements.push(element);
                    },
                    _ => {
                        todo!()
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#the-before-head-insertion-mode
            InsertionMode::BeforeHead => {
                match token {
                    Token::Character(TAB | LINE_FEED | FORM_FEED | WHITESPACE) => {
                        // Ignore the token.
                    },
                    Token::Comment(data) => {
                        // Insert a comment.
                        self.insert_comment(data);
                    },
                    Token::DOCTYPE(_) => {
                        // parse error, ignore token
                    },
                    Token::Tag(ref tagdata) if tagdata.opening && tagdata.name == "html" => {
                        // Process the token using the rules for the "in body" insertion mode.
                        self.consume_in_mode(InsertionMode::InBody, token);
                    },
                    Token::Tag(ref tagdata) if tagdata.opening && tagdata.name == "head" => {
                        todo!()
                    },
                    Token::Tag(ref tagdata)
                        if !tagdata.opening
                            && tagdata.name != "head"
                            && tagdata.name != "body"
                            && tagdata.name != "html"
                            && tagdata.name != "br" =>
                    {
                        // Parse error. Ignore the token.
                    },
                    _ => {
                        todo!()
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inhead
            InsertionMode::InHead => {
                match token {
                    Token::Character(c @ (TAB | LINE_FEED | FORM_FEED | WHITESPACE)) => {
                        todo!()
                    },
                    Token::Comment(data) => {
                        // Insert a comment.
                        self.insert_comment(data);
                    },
                    Token::DOCTYPE(_) => {
                        // parse error, ignore token
                    },
                    Token::Tag(ref tagdata) if tagdata.opening && tagdata.name == "html" => {
                        // Process the token using the rules for the "in body" insertion mode.
                        self.consume_in_mode(InsertionMode::InBody, token);
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening
                            && (tagdata.name == "base"
                                || tagdata.name == "basefont"
                                || tagdata.name == "bgsound"
                                || tagdata.name == "link") =>
                    {
                        // Insert an HTML element for the token.
                        self.insert_html_element_for_token(&tagdata);

                        // Immediately pop the current node off the stack of open elements.
                        self.open_elements.pop();

                        // Acknowledge the token's self-closing flag, if it is set.
                    },
                    Token::Tag(ref tagdata) if tagdata.opening && tagdata.name == "meta" => {
                        // Insert an HTML element for the token.
                        self.insert_html_element_for_token(tagdata);

                        // Immediately pop the current node off the stack of open elements.
                        self.open_elements.pop();

                        // Acknowledge the token's self-closing flag, if it is set.

                        // If the active speculative HTML parser is null, then:
                        //
                        //     If the element has a charset attribute, and getting an encoding
                        //     from its value results in an encoding, and the confidence is
                        //     currently tentative, then change the encoding to the resulting
                        //     encoding.
                        //
                        //     Otherwise, if the element has an http-equiv attribute whose
                        //     value is an ASCII case-insensitive match for the string
                        //     "Content-Type", and the element has a content attribute, and
                        //     applying the algorithm for extracting a character encoding
                        //     from a meta element to that attribute's value returns an
                        //     encoding, and the confidence is currently tentative, then
                        //     change the encoding to the extracted encoding.
                    },
                    Token::Tag(tagdata) if tagdata.opening && tagdata.name == "title" => {
                        // Follow the generic RCDATA element parsing algorithm.
                        self.generic_parsing_algorithm(tagdata, GenericParsingAlgorithm::RCDATA);
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening && tagdata.name == "noscript" && self.scripting =>
                    {
                        // Follow the generic raw text element parsing algorithm.
                        self.generic_parsing_algorithm(tagdata, GenericParsingAlgorithm::RawText);
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening
                            && (tagdata.name == "noframes" || tagdata.name == "style") =>
                    {
                        // Follow the generic raw text element parsing algorithm.
                        self.generic_parsing_algorithm(tagdata, GenericParsingAlgorithm::RawText);
                    },
                    Token::Tag(ref tagdata)
                        if tagdata.opening && tagdata.name == "noscript" && !self.scripting =>
                    {
                        // Insert an HTML element for the token.
                        self.insert_html_element_for_token(tagdata);

                        // Switch the insertion mode to "in head noscript".
                        self.insertion_mode = InsertionMode::InHeadNoscript;
                    },
                    Token::Tag(ref tagdata) if tagdata.opening && tagdata.name == "script" => {
                        unimplemented!();
                    },
                    Token::Tag(ref tagdata) if !tagdata.opening && tagdata.name == "head" => {
                        todo!()
                    },
                    Token::Tag(ref tagdata) if tagdata.opening && tagdata.name == "template" => {
                        // Insert an HTML element for the token.
                        self.insert_html_element_for_token(tagdata);

                        // Insert a marker at the end of the list of active formatting
                        // elements.
                        todo!();

                        // Set the frameset-ok flag to "not ok".
                        // todo!();

                        // Switch the insertion mode to "in template".
                        // self.insertion_mode = InsertionMode::InTemplate;

                        // Push "in template" onto the stack of template insertion modes so
                        // that it is the new current template insertion mode.
                        // todo!();
                    },
                    Token::Tag(ref tagdata) if !tagdata.opening && tagdata.name == "template" => {
                        // If there is no template element on the stack of open elements, then
                        // this is a parse error; ignore the token.
                        //
                        // Otherwise, run these steps:
                        //
                        //     Generate all implied end tags thoroughly.
                        //
                        //     If the current node is not a template element,
                        //     then this is a parse error.
                        //
                        //     Pop elements from the stack of open
                        //     elements until a template element has
                        //     been popped from the stack.
                        //
                        //     Clear the list of active
                        //     formatting elements up to the last marker.
                        //
                        //     Pop the current template
                        //     insertion mode off the stack of template
                        //     insertion modes.
                        //
                        //     Reset the insertion mode appropriately.
                        todo!();
                    },
                    Token::Tag(ref tagdata)
                        if (tagdata.opening && tagdata.name == "head")
                            || (!tagdata.opening
                                && tagdata.name != "body"
                                && tagdata.name != "html"
                                && tagdata.name != "br") =>
                    {
                        // Parse error. Ignore the token.
                    },
                    _ => {
                        todo!()
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inheadnoscript
            InsertionMode::InHeadNoscript => {
                match token {
                    Token::DOCTYPE(_) => {
                        // Parse error. Ignore the token.
                    },
                    Token::Tag(ref tagdata) if tagdata.opening && tagdata.name == "html" => {
                        // Process the token using the rules for the "in body" insertion
                        // mode.
                        self.consume_in_mode(InsertionMode::InBody, token);
                    },
                    Token::Tag(ref tagdata) if !tagdata.opening && tagdata.name == "noscript" => {
                        todo!()
                    },
                    Token::Character(TAB | LINE_FEED | FORM_FEED | WHITESPACE)
                    | Token::Comment(_) => {
                        // Process the token using the rules for the "in head" insertion
                        // mode.
                        self.consume_in_mode(InsertionMode::InHead, token);
                    },
                    Token::Tag(ref tagdata)
                        if tagdata.opening
                            && (tagdata.name == "basefont"
                                || tagdata.name == "bgsound"
                                || tagdata.name == "link"
                                || tagdata.name == "meta"
                                || tagdata.name == "style") =>
                    {
                        // Process the token using the rules for the "in head" insertion
                        // mode.
                        self.consume_in_mode(InsertionMode::InHead, token);
                    },
                    Token::Tag(ref tagdata)
                        if tagdata.opening
                            && (tagdata.name == "head" || tagdata.name == "noscript") => {}, // Parse error. Ignore the token.
                    Token::Tag(ref tagdata) if !tagdata.opening && (tagdata.name != "br") => {}, // Parse error. Ignore the token.
                    _ => {
                        todo!()
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#the-after-head-insertion-mode
            InsertionMode::AfterHead => {
                match token {
                    Token::Character(c @ (TAB | LINE_FEED | FORM_FEED | WHITESPACE)) => {
                        todo!()
                    },
                    Token::Comment(data) => {
                        // Insert a comment.
                        self.insert_comment(data);
                    },
                    Token::DOCTYPE(_) => {}, // Parse error. Ignore the token.
                    Token::Tag(ref tagdata) if tagdata.opening && tagdata.name == "html" => {
                        // Process the token using the rules for the "in body" insertion mode.
                        self.consume_in_mode(InsertionMode::InBody, token);
                    },
                    Token::Tag(tagdata) if tagdata.opening && tagdata.name == "body" => {
                        // Insert an HTML element for the token.
                        self.insert_html_element_for_token(&tagdata);

                        // Set the frameset-ok flag to "not ok".

                        // Switch the insertion mode to "in body".
                        self.insertion_mode = InsertionMode::InBody;
                    },
                    Token::Tag(tagdata) if tagdata.opening && tagdata.name == "frameset" => {
                        // Insert an HTML element for the token.
                        self.insert_html_element_for_token(&tagdata);

                        // Switch the insertion mode to "in frameset".
                        self.insertion_mode = InsertionMode::InFrameset;
                    },
                    Token::Tag(ref tagdata)
                        if tagdata.opening
                            && (tagdata.name == "base"
                                || tagdata.name == "basefont"
                                || tagdata.name == "bgsound"
                                || tagdata.name == "link"
                                || tagdata.name == "meta"
                                || tagdata.name == "noframes"
                                || tagdata.name == "script"
                                || tagdata.name == "style"
                                || tagdata.name == "template"
                                || tagdata.name == "title") =>
                    {
                        todo!()
                    },
                    Token::Tag(ref tagdata) if !tagdata.opening && tagdata.name == "template" => {
                        // Process the token using the rules for the "in head" insertion mode.
                        self.consume_in_mode(InsertionMode::InHead, token);
                    },
                    Token::Tag(tagdata) if tagdata.opening && tagdata.name == "head" => {}, // Parse error. Ignore the token.
                    Token::Tag(tagdata)
                        if !tagdata.opening
                            && tagdata.name != "body"
                            && tagdata.name != "html"
                            && tagdata.name != "br" => {}, // Parse error. Ignore the token.
                    _ => {
                        // Insert an HTML element for a "body" start tag token with no attributes.
                        let body_token = TagData {
                            opening: true,
                            name: "body".to_string(),
                            self_closing: false,
                            attributes: vec![],
                        };
                        self.insert_html_element_for_token(&body_token);

                        // Switch the insertion mode to "in body".
                        self.insertion_mode = InsertionMode::InBody;

                        // Reprocess the current token.
                        self.consume(token);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inbody
            InsertionMode::InBody => {
                match token {
                    Token::Character('\0') => {
                        // Parse error. Ignore the token.
                    },
                    Token::Character(c) => {
                        todo!()
                    },
                    Token::Comment(data) => {
                        // Insert a comment.
                        self.insert_comment(data);
                    },
                    Token::DOCTYPE(_) => {}, // Parse error. Ignore the token.
                    Token::Tag(tagdata) if tagdata.opening && tagdata.name == "html" => {
                        // Parse error.

                        // If there is a template element on the stack of open elements, then ignore
                        // the token.

                        // Otherwise, for each attribute on the token, check to see if the attribute is
                        // already present on the top element of the stack of open elements. If it is
                        // not, add the attribute and its corresponding value to that element.
                        todo!();
                    },
                    Token::Tag(ref tagdata)
                        if (tagdata.opening
                            && (tagdata.name == "base"
                                || tagdata.name == "basefont"
                                || tagdata.name == "bgsound"
                                || tagdata.name == "link"
                                || tagdata.name == "meta"
                                || tagdata.name == "noframes"
                                || tagdata.name == "script"
                                || tagdata.name == "style"
                                || tagdata.name == "template"
                                || tagdata.name == "title"))
                            || (!tagdata.opening && tagdata.name == "template") =>
                    {
                        // Process the token using the rules for the "in head" insertion mode.
                        self.consume_in_mode(InsertionMode::InHead, token);
                    },
                    Token::Tag(ref tagdata) if tagdata.opening && tagdata.name == "body" => {
                        // Parse error.

                        // If the second element on the stack of open elements is not a body element,
                        // if the stack of open elements has only one node on it, or if there is a
                        // template element on the stack of open elements, then ignore the token.
                        // (fragment case)

                        // Otherwise, set the frameset-ok flag to "not ok"; then, for each attribute on
                        // the token, check to see if the attribute is already present on the body
                        // element (the second element) on the stack of open elements, and if it is
                        // not, add the attribute and its corresponding value to that element.
                        todo!();
                    },
                    Token::Tag(ref tagdata) if tagdata.opening && tagdata.name == "frameset" => {
                        todo!()
                    },
                    Token::EOF => {
                        // If the stack of template insertion modes is not empty, then process the
                        // token using the rules for the "in template" insertion mode.
                        // FIXME we don't have a template insertion mode yet

                        // Otherwise, follow these steps:
                        //     If there is a node in the stack of open elements that is not either
                        //     a dd element, a dt element, an li element, an optgroup element, an
                        //     option element, a p element, an rb element, an rp element, an rt
                        //     element, an rtc element, a tbody element, a td element, a tfoot
                        //     element, a th element, a thead element, a tr element, the body
                        //     element, or the html element, then this is a parse error.
                        //
                        //     Stop parsing.
                        self.done = true;
                    },
                    Token::Tag(ref tagdata) if !tagdata.opening && tagdata.name == "body" => {
                        todo!()
                    },

                    Token::Tag(ref tagdata) if !tagdata.opening && tagdata.name == "body" => {
                        todo!()
                    },
                    Token::Tag(ref tagdata)
                        if tagdata.opening
                            && (tagdata.name == "address"
                                || tagdata.name == "article"
                                || tagdata.name == "aside"
                                || tagdata.name == "blockquote"
                                || tagdata.name == "center"
                                || tagdata.name == "details"
                                || tagdata.name == "dialog"
                                || tagdata.name == "dir"
                                || tagdata.name == "div"
                                || tagdata.name == "dl"
                                || tagdata.name == "fieldset"
                                || tagdata.name == "figcaption"
                                || tagdata.name == "figure"
                                || tagdata.name == "footer"
                                || tagdata.name == "header"
                                || tagdata.name == "hgroup"
                                || tagdata.name == "main"
                                || tagdata.name == "menu"
                                || tagdata.name == "nav"
                                || tagdata.name == "ol"
                                || tagdata.name == "p"
                                || tagdata.name == "section"
                                || tagdata.name == "summary"
                                || tagdata.name == "ul") =>
                    {
                        todo!()
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening
                            && (tagdata.name == "h1"
                                || tagdata.name == "h2"
                                || tagdata.name == "h3"
                                || tagdata.name == "h4"
                                || tagdata.name == "h5"
                                || tagdata.name == "h6") =>
                    {
                        todo!()
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening
                            && (tagdata.name == "pre" || tagdata.name == "listing") =>
                    {
                        todo!()
                    },
                    Token::Tag(tagdata) if tagdata.opening && tagdata.name == "form" => {
                        todo!()
                    },
                    Token::Tag(tagdata) if tagdata.opening && tagdata.name == "li" => {
                        todo!("handle li")
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening && (tagdata.name == "dd" || tagdata.name == "dt") =>
                    {
                        todo!("handle dd/dt")
                    },
                    Token::Tag(tagdata) if tagdata.opening && tagdata.name == "plaintext" => {
                        todo!()
                    },
                    Token::Tag(tagdata) if tagdata.opening && tagdata.name == "button" => {
                        todo!()
                    },
                    Token::Tag(ref tagdata)
                        if tagdata.opening
                            && (tagdata.name == "address"
                                || tagdata.name == "article"
                                || tagdata.name == "aside"
                                || tagdata.name == "blockquote"
                                || tagdata.name == "center"
                                || tagdata.name == "details"
                                || tagdata.name == "dialog"
                                || tagdata.name == "dir"
                                || tagdata.name == "div"
                                || tagdata.name == "dl"
                                || tagdata.name == "fieldset"
                                || tagdata.name == "figcaption"
                                || tagdata.name == "figure"
                                || tagdata.name == "footer"
                                || tagdata.name == "header"
                                || tagdata.name == "hgroup"
                                || tagdata.name == "main"
                                || tagdata.name == "menu"
                                || tagdata.name == "nav"
                                || tagdata.name == "ol"
                                || tagdata.name == "p"
                                || tagdata.name == "section"
                                || tagdata.name == "summary"
                                || tagdata.name == "ul") =>
                    {
                        todo!()
                    },
                    Token::Tag(ref tagdata) if !tagdata.opening && tagdata.name == "form" => {
                        todo!()
                    },
                    Token::Tag(ref tagdata) if !tagdata.opening && tagdata.name == "p" => {
                        todo!()
                    },
                    Token::Tag(ref tagdata) if !tagdata.opening && tagdata.name == "li" => {
                        todo!("handle li closing tag")
                    },
                    Token::Tag(ref tagdata)
                        if !tagdata.opening && (tagdata.name == "dd" || tagdata.name == "dt") =>
                    {
                        todo!("handle dd/dt closing tag")
                    },
                    Token::Tag(ref tagdata)
                        if !tagdata.opening
                            && (tagdata.name == "h1"
                                || tagdata.name == "h2"
                                || tagdata.name == "h3"
                                || tagdata.name == "h4"
                                || tagdata.name == "h5"
                                || tagdata.name == "h6") =>
                    {
                        todo!()
                    },
                    Token::Tag(ref tagdata) if tagdata.opening && tagdata.name == "a" => {
                        todo!("handle a tag opening")
                    },

                    // FIXME a lot of (for now) irrelevant rules are missing here
                    Token::Tag(tagdata) if tagdata.opening => {
                        todo!()
                    },

                    _ => todo!(),
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-afterbody
            InsertionMode::AfterBody => {
                match token {
                    Token::Character(TAB | LINE_FEED | FORM_FEED | WHITESPACE) => {
                        // Process the token using the rules for the "in body" insertion mode.
                        self.consume_in_mode(InsertionMode::InBody, token);
                    },
                    Token::Comment(data) => {
                        // Insert a comment as the last child of the first element in the stack of
                        // open elements (the html element).
                        self.insert_comment(data)
                    },
                    Token::DOCTYPE(_) => {
                        // parse error, ignore token
                    },
                    Token::Tag(ref tagdata) if tagdata.opening && tagdata.name == "html" => {
                        // Process the token using the rules for the "in body" insertion mode.
                        self.consume_in_mode(InsertionMode::InBody, token);
                    },
                    Token::Tag(tagdata) if !tagdata.opening && tagdata.name == "html" => {
                        // FIXME
                        // If the parser was created as part of the HTML fragment parsing
                        // algorithm, this is a parse error; ignore the token. (fragment case)

                        // Otherwise, switch the insertion mode to "after after body".
                        self.insertion_mode = InsertionMode::AfterAfterBody;
                    },
                    Token::EOF => {
                        // Stop parsing.
                        self.done = true;
                    },
                    _ => {
                        // Parse error. Switch the insertion mode to "in body" and reprocess the
                        // token.
                        self.insertion_mode = InsertionMode::InBody;
                        self.consume(token);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#the-after-after-body-insertion-mode
            InsertionMode::AfterAfterBody => {
                match token {
                    Token::Comment(data) => {
                        // Insert a comment as the last child of the Document object. FIXME is the
                        // first element the document?
                        self.insert_comment(data);
                    },
                    Token::Character(TAB | LINE_FEED | FORM_FEED | WHITESPACE)
                    | Token::DOCTYPE(_) => {
                        // Process the token using the rules for the "in body" insertion mode.
                        self.consume_in_mode(InsertionMode::InBody, token);
                    },
                    Token::Tag(ref tagdata) if tagdata.opening && tagdata.name == "html" => {
                        // Process the token using the rules for the "in body" insertion mode.
                        self.consume_in_mode(InsertionMode::InBody, token);
                    },
                    Token::EOF => {
                        // Stop parsing.
                        self.done = true;
                    },
                    _ => {
                        // Parse error. Switch the insertion mode to "in body" and reprocess the
                        // token.
                        self.insertion_mode = InsertionMode::InBody;
                        self.consume(token);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-incdata
            InsertionMode::Text => {
                match token {
                    Token::Character(_) => {
                        // Insert the token's character.
                    },
                    Token::Tag(ref tagdata) if !tagdata.opening => {
                        // Pop the current node off the stack of open elements.
                        self.open_elements.pop();

                        // Switch the insertion mode to the original insertion mode.
                        self.insertion_mode = self.original_insertion_mode.unwrap();
                    },
                    _ => todo!(),
                }
            },
            _ => todo!("Implement '{mode:?}' state"),
        }
    }
}
