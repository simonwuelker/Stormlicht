//! Implements the [Tree Construction Stage](https://html.spec.whatwg.org/multipage/parsing.html#tree-construction)

use crate::dom::{DOMNode, DOMNodeType, SharedDOMNode};
use crate::tokenizer::{TagData, Token, Tokenizer, TokenizerState};

const DEFAULT_SCOPE: Vec<DOMNodeType> = vec![]; // FIXME
const BUTTON_SCOPE: Vec<DOMNodeType> = vec![]; // FIXME

const TAB: char = '\u{0009}';
const LINE_FEED: char = '\u{000A}';
const FORM_FEED: char = '\u{000C}';
const SPACE: char = '\u{0020}';

const HEADINGS: [DOMNodeType; 6] = [
    DOMNodeType::H1,
    DOMNodeType::H2,
    DOMNodeType::H3,
    DOMNodeType::H4,
    DOMNodeType::H5,
    DOMNodeType::H6,
];

#[derive(Debug, Clone, Copy)]
/// https://html.spec.whatwg.org/multipage/parsing.html#parse-state
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
    open_elements: Vec<SharedDOMNode>,
    head: Option<SharedDOMNode>,
    scripting: bool,
    done: bool,
}

impl<'source> Parser<'source> {
    pub fn new(source: &'source str) -> Self {
        Self {
            tokenizer: Tokenizer::new(source),
            original_insertion_mode: None,
            insertion_mode: InsertionMode::Initial,
            open_elements: vec![DOMNode::new(DOMNodeType::Document).to_shared()],
            head: None,
            scripting: false,
            done: false,
        }
    }

    pub fn parse(mut self) -> SharedDOMNode {
        while let Some(token) = self.tokenizer.next() {
            self.consume(token);

            if self.done {
                break;
            }
        }
        return self.open_elements[0].clone();
    }

    fn current_node(&self) -> &SharedDOMNode {
        // safe because self.open_elements can never be empty
        self.open_elements.last().unwrap()
    }

    fn consume(&mut self, token: Token) {
        self.consume_in_mode(self.insertion_mode, token);
    }

    // https://html.spec.whatwg.org/multipage/parsing.html#creating-and-inserting-nodes
    fn appropriate_place_for_inserting_node(&self) -> SharedDOMNode {
        self.appropriate_place_for_inserting_node_with_override(None)
    }

    // https://html.spec.whatwg.org/multipage/parsing.html#creating-and-inserting-nodes
    fn appropriate_place_for_inserting_node_with_override(
        &self,
        override_target: Option<SharedDOMNode>,
    ) -> SharedDOMNode {
        // If there was an override target specified, then let target be the override target.
        // Otherwise, let target be the current node.
        let target = match override_target {
            Some(ref target) => target,
            None => self.current_node(),
        };

        // TODO spec talks about foster parenting here, which we don't support

        // Let adjusted insertion location be inside target, after its last child (if any).
        let adjusted_insertion_location = target;

        // TODO
        // If the adjusted insertion location is inside a template element, let it instead be
        // inside the template element's template contents, after its last child (if any).

        // Return the adjusted insertion location.
        adjusted_insertion_location.clone()
    }

    // https://html.spec.whatwg.org/multipage/parsing.html#insert-a-comment
    fn insert_comment_at(&mut self, data: String, at: Option<SharedDOMNode>) {
        // Let data be the data given in the comment token being processed.

        // If position was specified, then let the adjusted insertion location be position. Otherwise,
        // let adjusted insertion location be the appropriate place for inserting a node.
        let adjusted_insert_location = match at {
            Some(location) => location,
            None => self.appropriate_place_for_inserting_node(),
        };

        // Create a Comment node whose data attribute is set to data and whose node document is
        // the same as that of the node in which the adjusted insertion location finds itself.

        // insert the newly created node at the adjusted insertion location.
        DOMNode::add_child(
            adjusted_insert_location,
            DOMNode::new(DOMNodeType::Comment { data: data }).to_shared(),
        )
    }

    // https://html.spec.whatwg.org/multipage/parsing.html#insert-a-comment
    fn insert_comment(&mut self, data: String) {
        self.insert_comment_at(data, None);
    }

    // https://html.spec.whatwg.org/multipage/parsing.html#parsing-elements-that-contain-only-text
    fn generic_rcdata_element_parsing_algorithm(&mut self, tagdata: TagData) {
        // Insert an HTML element for the token.
        let domnode = DOMNode::from(tagdata).to_shared();
        DOMNode::add_child(self.current_node().clone(), domnode);

        // switch the tokenizer to the RCDATA state.
        self.tokenizer.switch_to(TokenizerState::RCDATAState);

        // Let the original insertion mode be the current insertion mode.
        self.original_insertion_mode = Some(self.insertion_mode);

        // Then, switch the insertion mode to "text".
        self.insertion_mode = InsertionMode::Text;
    }

    // https://html.spec.whatwg.org/multipage/parsing.html#parsing-elements-that-contain-only-text
    fn generic_raw_text_element_parsing_algorithm(&mut self, tagdata: TagData) {
        // Insert an HTML element for the token.
        let domnode = DOMNode::from(tagdata).to_shared();
        DOMNode::add_child(self.current_node().clone(), domnode);

        // switch the tokenizer to the RAWTEXT state.
        self.tokenizer.switch_to(TokenizerState::RAWTEXTState);

        // Let the original insertion mode be the current insertion mode.
        self.original_insertion_mode = Some(self.insertion_mode);

        // Then, switch the insertion mode to "text".
        self.insertion_mode = InsertionMode::Text;
    }

    fn is_element_in_scope(&self, target_node_type: &DOMNodeType, scope: Vec<DOMNodeType>) -> bool {
        // The stack of open elements is said to have an element target node in a specific scope
        // consisting of a list of element types list when the following algorithm terminates in a
        // match state:

        // Initialize node to be the current node (the bottommost node of the stack).
        let mut index = self.open_elements.len() - 1;
        loop {
            let node = &self.open_elements[index];
            let node_type = &node.borrow().node_type;
            // If node is the target node
            if node_type == target_node_type {
                // , terminate in a match state.
                return true;
            }
            // Otherwise, if node is one of the element types in list
            else if scope.contains(&node_type) {
                // , terminate in a failure state.
                return false;
            } else {
                // Otherwise, set node to the previous entry in the stack of open elements and
                // return to step 2. (This will never fail, since the loop will always terminate in
                // the previous step if the top of the stack — an html element — is reached.)
                index -= 1;
            }
        }
    }

    fn close_p_element(&mut self) {
        self.generate_implied_end_tags_excluding(Some(DOMNodeType::P));
        // Pop elements from the stack of open elements until a p element has been popped from the
        // stack.
        while let Some(node) = self.open_elements.pop() {
            if matches!(node.borrow().node_type, DOMNodeType::P) {
                break;
            }
        }
    }

    // https://html.spec.whatwg.org/multipage/parsing.html#closing-elements-that-have-implied-end-tags
    fn generate_implied_end_tags(&mut self) {
        self.generate_implied_end_tags_excluding(None);
    }

    // https://html.spec.whatwg.org/multipage/parsing.html#closing-elements-that-have-implied-end-tags
    fn generate_implied_end_tags_excluding(&mut self, exclude: Option<DOMNodeType>) {
        // pop as long as the node type is in a give set of types excluding an (optional) value
        loop {
            {
                let node_type = &self.current_node().borrow().node_type;
                if exclude.contains(node_type) || !vec![].contains(node_type) {
                    break;
                }
            }
            self.open_elements.pop().unwrap();
        }
    }

    // https://html.spec.whatwg.org/multipage/parsing.html#create-an-element-for-the-token
    fn create_html_element_for_token(&self, tagdata: &TagData) -> SharedDOMNode {
        // TODO there's a long algorithm here and we don't support 99% of it. Implement the rest
        let mut html_element = DOMNode::new(DOMNodeType::from(tagdata.name.as_str()));

        // Append each attribute in the given token to element.
        html_element.attributes = tagdata.attributes.clone();

        html_element.to_shared()
    }

    fn insert_html_element_for_token(&mut self, tagdata: &TagData) -> SharedDOMNode {
        let html_element = DOMNode::new(DOMNodeType::from(tagdata.name.as_str())).to_shared();
        DOMNode::add_child(self.current_node().clone(), html_element.clone());
        self.open_elements.push(html_element.clone());
        html_element
    }

    // https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inhtml
    fn consume_in_mode(&mut self, mode: InsertionMode, token: Token) {
        match mode {
            // https://html.spec.whatwg.org/multipage/parsing.html#the-initial-insertion-mode
            InsertionMode::Initial => {
                match token {
                    Token::Character(TAB | LINE_FEED | FORM_FEED | SPACE) => {
                        // Ignore the token.
                    },
                    Token::Comment(data) => {
                        // Insert a comment as the last child of the Document object.
                        self.insert_comment(data);
                    },
                    Token::DOCTYPE(doctype) => {
                        let domnode_type = DOMNodeType::DocumentType {
                            name: doctype.name.unwrap_or_default(),
                            public_ident: doctype.public_ident.unwrap_or_default(),
                            system_ident: doctype.system_ident.unwrap_or_default(),
                        };
                        // TODO: check if we need to set document to quirks mode
                        // https://html.spec.whatwg.org/multipage/parsing.html#the-before-html-insertion-mode
                        DOMNode::add_child(
                            self.current_node().clone(),
                            DOMNode::new(domnode_type).to_shared(),
                        );
                        self.insertion_mode = InsertionMode::BeforeHtml;
                    },
                    _ => {
                        // TODO check for quirks mode

                        // In any case, switch the insertion mode to "before html", then reprocess the token.
                        self.insertion_mode = InsertionMode::BeforeHtml;
                        self.consume(token);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#the-before-html-insertion-mode
            InsertionMode::BeforeHtml => {
                match token {
                    Token::Character(TAB | LINE_FEED | FORM_FEED | SPACE) => {
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
                        let dom_element = self.create_html_element_for_token(tagdata);

                        // Append it to the Document object.
                        DOMNode::add_child(self.current_node().clone(), dom_element.clone());

                        // Put this element in the stack of open elements.
                        self.open_elements.push(dom_element);

                        // Switch the insertion mode to "before head".
                        self.insertion_mode = InsertionMode::BeforeHead;
                    },
                    _ => {
                        // Create an html element whose node document is the Document object.
                        let dom_element = DOMNode::new(DOMNodeType::Html).to_shared();

                        // Append it to the Document object.
                        DOMNode::add_child(self.current_node().clone(), dom_element.clone());

                        // Put this element in the stack of open elements.
                        self.open_elements.push(dom_element);

                        // Switch the insertion mode to "before head", then reprocess the token.
                        self.insertion_mode = InsertionMode::BeforeHead;
                        self.consume(token);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#the-before-head-insertion-mode
            InsertionMode::BeforeHead => {
                match token {
                    Token::Character(TAB | LINE_FEED | FORM_FEED | SPACE) => {
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
                        // Insert an HTML element for the token.
                        let head = self.insert_html_element_for_token(tagdata);

                        // Set the head element pointer to the newly created head element.
                        self.head = Some(head);

                        // Switch the insertion mode to "in head".
                        self.insertion_mode = InsertionMode::InHead;
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
                        // Insert an HTML element for a "head" start tag token with no attributes.
                        let head_tagdata = TagData {
                            opening: true,
                            name: "head".to_string(),
                            self_closing: false,
                            attributes: vec![],
                        };
                        let head = self.insert_html_element_for_token(&head_tagdata);

                        // Set the head element pointer to the newly created head element.
                        self.head = Some(head);

                        // Switch the insertion mode to "in head".
                        self.insertion_mode = InsertionMode::InHead;

                        // Reprocess the current token.
                        self.consume(token);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inhead
            InsertionMode::InHead => {
                match token {
                    Token::Character(c @ (TAB | LINE_FEED | FORM_FEED | SPACE)) => {
                        // Insert the character.
                        DOMNode::add_text(self.current_node(), c.to_string());
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
                        self.generic_rcdata_element_parsing_algorithm(tagdata);
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening && tagdata.name == "noscript" && self.scripting =>
                    {
                        // Follow the generic raw text element parsing algorithm.
                        self.generic_raw_text_element_parsing_algorithm(tagdata);
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening
                            && (tagdata.name == "noframes" || tagdata.name == "style") =>
                    {
                        // Follow the generic raw text element parsing algorithm.
                        self.generic_raw_text_element_parsing_algorithm(tagdata);
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
                        // Pop the current node (which will be the head element) off the stack of open elements.
                        let top_node = self.open_elements.pop();
                        assert!(top_node.is_some());
                        assert_eq!(top_node.unwrap().borrow().node_type, DOMNodeType::Head);

                        // Switch the insertion mode to "after head".
                        self.insertion_mode = InsertionMode::AfterHead;
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
                        // Pop the current node (which will be the head element) off the stack of open elements.
                        let top_node = self.open_elements.pop();
                        assert!(top_node.is_some());
                        assert_eq!(top_node.unwrap().borrow().node_type, DOMNodeType::Head);

                        // Switch the insertion mode to "after head".
                        self.insertion_mode = InsertionMode::AfterHead;

                        // Reprocess the token.
                        self.consume(token);
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
                        // Pop the current node (which will be a noscript element) from
                        // the stack of open elements; the new current node will be a
                        // head element.
                        let top_node = self.open_elements.pop();
                        assert!(top_node.is_some());
                        assert_eq!(top_node.unwrap().borrow().node_type, DOMNodeType::NoScript);
                        assert!(self.current_node().borrow().node_type == DOMNodeType::Head);

                        // Switch the insertion mode to "in head".
                        self.insertion_mode = InsertionMode::InHead;
                    },
                    Token::Character(TAB | LINE_FEED | FORM_FEED | SPACE) | Token::Comment(_) => {
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
                        // Parse error.

                        // Pop the current node (which will be a noscript element) from the stack of open elements;
                        // the new current node will be a head element.
                        let top_node = self.open_elements.pop();
                        assert!(top_node.is_some());
                        assert_eq!(top_node.unwrap().borrow().node_type, DOMNodeType::NoScript);
                        assert!(self.current_node().borrow().node_type == DOMNodeType::Head);

                        // Switch the insertion mode to "in head".
                        self.insertion_mode = InsertionMode::InHead;

                        // Reprocess the token.
                        self.consume(token);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#the-after-head-insertion-mode
            InsertionMode::AfterHead => {
                match token {
                    Token::Character(c @ (TAB | LINE_FEED | FORM_FEED | SPACE)) => {
                        // Insert the character.
                        DOMNode::add_text(self.current_node(), c.to_string());
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
                        // Parse error.
                        //
                        // Push the node pointed to by the head element pointer onto the stack
                        // of open elements.
                        assert!(self.head.is_some());
                        self.open_elements.push(self.head.clone().unwrap());

                        // Process the token using the rules for the "in head" insertion mode.
                        self.consume_in_mode(InsertionMode::InHead, token);

                        // Remove the node pointed to by the head element pointer from the
                        // stack of open elements. (It might not be the current node at this
                        // point.)
                        self.open_elements
                            .retain(|node| !matches!(node.borrow().node_type, DOMNodeType::Head));
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
                        // Reconstruct the active formatting elements, if any.

                        // Insert the character.
                        DOMNode::add_text(self.current_node(), c.to_string());
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
                        // Parse error.

                        // If the stack of open elements has only one node on it, or if the second
                        // element on the stack of open elements is not a body element
                        if self.open_elements.len() == 1
                            || self.open_elements.len() < 2
                            || !matches!(
                                self.open_elements[self.open_elements.len() - 2]
                                    .borrow()
                                    .node_type,
                                DOMNodeType::Body
                            )
                        {
                            // , then ignore the token. (fragment case)
                        }
                        // If the frameset-ok flag is set to "not ok", ignore the token.
                        todo!();
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
                        // If the stack of open elements does not have a body element in scope
                        if !self.is_element_in_scope(&DOMNodeType::Body, DEFAULT_SCOPE) {
                        }
                        // , this is a parse error; ignore the token.
                        else {
                            // Otherwise, if there is a node in the stack of open elements that is
                            // not either a dd element, a dt element, an li element, an optgroup
                            // element, an option element, a p element, an rb element, an rp
                            // element, an rt element, an rtc element, a tbody element, a td
                            // element, a tfoot element, a th element, a thead element, a tr
                            // element, the body element, or the html element, then this is a parse
                            // error.
                            // NOTE: since wen don't care about parse errors anyways, noop

                            // Switch the insertion mode to "after body".
                            self.insertion_mode = InsertionMode::AfterBody;
                        }
                    },

                    Token::Tag(ref tagdata) if !tagdata.opening && tagdata.name == "body" => {
                        // If the stack of open elements does not have a body element in scope
                        if !self.is_element_in_scope(&DOMNodeType::Body, DEFAULT_SCOPE) {
                        }
                        // , this is a parse error; ignore the token.
                        else {
                            // Otherwise, if there is a node in the stack of open elements that is
                            // not either a dd element, a dt element, an li element, an optgroup
                            // element, an option element, a p element, an rb element, an rp
                            // element, an rt element, an rtc element, a tbody element, a td
                            // element, a tfoot element, a th element, a thead element, a tr
                            // element, the body element, or the html element, then this is a parse
                            // error.
                            // NOTE: since wen don't care about parse errors anyways, noop

                            // Switch the insertion mode to "after body".
                            self.insertion_mode = InsertionMode::AfterBody;

                            // Reprocess the token.
                            self.consume(token);
                        }
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
                        todo!();
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
                        // If the stack of open elements has a p element in button scope
                        if self.is_element_in_scope(&DOMNodeType::P, BUTTON_SCOPE) {
                            // , then close a p element.
                            self.close_p_element();
                        }

                        // If the current node is an HTML element whose tag name is one of
                        // "h1", "h2", "h3", "h4", "h5", or "h6"
                        if HEADINGS
                            .iter()
                            .any(|h| h == &self.current_node().borrow().node_type)
                        {
                            // , then this is a parse error;
                            // pop the current node off the stack of open elements.
                            self.open_elements.pop();
                        }

                        // Insert an HTML element for the token.
                        self.insert_html_element_for_token(&tagdata);
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening
                            && (tagdata.name == "pre" || tagdata.name == "listing") =>
                    {
                        // If the stack of open elements has a p element in button scope
                        if self.is_element_in_scope(&DOMNodeType::P, BUTTON_SCOPE) {
                            // , then close a p element.
                            self.close_p_element();
                        }

                        // Insert an HTML element for the token.
                        self.insert_html_element_for_token(&tagdata);

                        // If the next token is a U+000A LINE FEED (LF) character token, then
                        // ignore that token and move on to the next one. (Newlines at the start of
                        // pre blocks are ignored as an authoring convenience.)
                        let next_token_or_none = self.tokenizer.next();
                        match next_token_or_none {
                            Some(Token::Character('\n')) | None => {},
                            Some(next_token) => self.consume(next_token),
                        }

                        // Set the frameset-ok flag to "not ok".
                        // FIXME we don't have a frameset-ok flag
                    },
                    Token::Tag(tagdata) if tagdata.opening && tagdata.name == "form" => {
                        // If the form element pointer is not null, and there is no template
                        // element on the stack of open elements, then this is a parse error;
                        // ignore the token.
                        // FIXME

                        // If the stack of open elements has a p element in button scope
                        if self.is_element_in_scope(&DOMNodeType::P, BUTTON_SCOPE) {
                            // , then close a p element.
                            self.close_p_element();
                        }

                        // Insert an HTML element for the token
                        self.insert_html_element_for_token(&tagdata);

                        // if there is no template element on the stack of open elements, set the
                        // form element pointer to point to the element created.
                        // FIXME
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
                        // If the stack of open elements has a p element in button scope
                        if self.is_element_in_scope(&DOMNodeType::P, BUTTON_SCOPE) {
                            // , then close a p element.
                            self.close_p_element();
                        }

                        // Insert an HTML element for the token.
                        self.insert_html_element_for_token(&tagdata);

                        // Switch the tokenizer to the PLAINTEXT state.
                        // NOTE: Once a start tag with the tag name "plaintext" has been seen, that will
                        // be the last token ever seen other than character tokens (and the end-of-file
                        // token), because there is no way to switch out of the PLAINTEXT state.
                        self.tokenizer.switch_to(TokenizerState::PLAINTEXTState);
                    },
                    Token::Tag(tagdata) if tagdata.opening && tagdata.name == "button" => {
                        // If the stack of open elements has a button element in scope
                        if self.is_element_in_scope(&DOMNodeType::Button, DEFAULT_SCOPE) {
                            // Parse error.

                            // Generate implied end tags.
                            self.generate_implied_end_tags();

                            // Pop elements from the stack of open elements until a button element has
                            // been popped from the stack.
                            while let Some(node) = self.open_elements.pop() {
                                if matches!(node.borrow().node_type, DOMNodeType::Button) {
                                    break;
                                }
                            }
                        }
                        // Reconstruct the active formatting elements, if any.
                        // FIXME

                        // Insert an HTML element for the token.
                        self.insert_html_element_for_token(&tagdata);

                        // Set the frameset-ok flag to "not ok".
                        // FIXME
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
                        // If the stack of open elements does not have a p element in button scope
                        if !self.is_element_in_scope(&DOMNodeType::P, DEFAULT_SCOPE) {
                            // , then this is a parse error; insert an HTML element for a "p" start
                            // tag token with no attributes.
                            let p_tagdata = TagData {
                                opening: true,
                                name: "p".to_string(),
                                self_closing: false,
                                attributes: vec![],
                            };
                            self.insert_html_element_for_token(&p_tagdata);
                        }

                        // Close a p element.
                        self.close_p_element();
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
                        // If the stack of open elements does not have an element in scope that is
                        // an HTML element and whose tag name is one of "h1", "h2", "h3", "h4",
                        // "h5", or "h6"
                        let required = DOMNodeType::from(tagdata.name.as_str());
                        if !self.is_element_in_scope(&required, DEFAULT_SCOPE) {
                            // , then this is a parse error; ignore the token.
                        } else {
                            // Generate implied end tags.
                            self.generate_implied_end_tags();

                            // If the current node is not an HTML element with the same tag name as
                            // that of the token, then this is a parse error.
                            // FIXME i don't really care about parse errors right now

                            // Pop elements from the stack of open elements until an HTML element
                            // whose tag name is one of "h1", "h2", "h3", "h4", "h5", or "h6" has
                            // been popped from the stack.
                            while let Some(node) = self.open_elements.pop() {
                                if node.borrow().node_type == required {
                                    break;
                                }
                            }
                        }
                    },
                    Token::Tag(ref tagdata) if tagdata.opening && tagdata.name == "a" => {
                        todo!("handle a tag opening")
                    },

                    // FIXME a lot of (for now) irrelevant rules are missing here
                    Token::Tag(tagdata) if tagdata.opening => {
                        // Reconstruct the active formatting elements, if any.
                        // FIXME

                        let domnode = DOMNode::from(tagdata).to_shared();
                        DOMNode::add_child(self.current_node().clone(), domnode.clone());
                        self.open_elements.push(domnode);
                    },

                    _ => todo!(),
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-afterbody
            InsertionMode::AfterBody => {
                match token {
                    Token::Character(TAB | LINE_FEED | FORM_FEED | SPACE) => {
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
                    Token::Character(TAB | LINE_FEED | FORM_FEED | SPACE) | Token::DOCTYPE(_) => {
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
            _ => todo!("Implement '{mode:?}' state"),
        }
    }
}
