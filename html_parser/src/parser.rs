use crate::dom::{DOMNode, DOMNodeType, SharedDOMNode};
use crate::tokenizer::{TagData, Token, Tokenizer, TokenizerState};


#[derive(Debug, Clone, Copy)]
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
        }
    }

    pub fn parse(mut self) {
        while let Some(token) = self.tokenizer.next() {
            self.consume(token);
        }
    }

    fn current_node(&self) -> &SharedDOMNode {
        // safe because self.open_elements can never be empty
        self.open_elements.last().unwrap()
    }

    fn consume(&mut self, token: Token) {
        self.consume_in_mode(self.insertion_mode, token);
    }

    fn consume_in_mode(&mut self, mode: InsertionMode, token: Token) {
        match mode {
            InsertionMode::Initial => {
                match token {
                    //                   tab       line feed    form feed     space
                    Token::Character('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}') => {}
                    Token::Comment(data) => DOMNode::add_child(
                        self.current_node().clone(),
                        DOMNode::new(DOMNodeType::Comment { data: data }).to_shared(),
                    ),
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
                    }
                    _ => {
                        // TODO check for quirks mode
                        self.insertion_mode = InsertionMode::BeforeHtml;
                        self.consume(token); // reconsume
                    }
                }
            }
            InsertionMode::BeforeHtml => {
                match &token {
                    //                   tab       line feed    form feed     space
                    Token::Character('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}') => {}
                    Token::Comment(data) => DOMNode::add_child(
                        self.current_node().clone(),
                        DOMNode::new(DOMNodeType::Comment {
                            data: data.to_string(),
                        })
                        .to_shared(),
                    ),
                    Token::DOCTYPE(_) => {} // parse error, ignore token
                    Token::Tag(tagdata)
                        if !tagdata.opening
                            && tagdata.name != "head"
                            && tagdata.name != "body"
                            && tagdata.name != "html"
                            && tagdata.name != "br" => {} // Parse error. Ignore the token.
                    Token::Tag(tagdata) if tagdata.opening && tagdata.name == "html" => {
                        let dom_element = DOMNode::new(DOMNodeType::Html).to_shared();
                        DOMNode::add_child(self.current_node().clone(), dom_element.clone());
                        self.open_elements.push(dom_element);
                        self.insertion_mode = InsertionMode::BeforeHead;
                    }
                    _ => {
                        let dom_element = DOMNode::new(DOMNodeType::Html).to_shared();
                        DOMNode::add_child(self.current_node().clone(), dom_element.clone());
                        self.open_elements.push(dom_element);
                        self.insertion_mode = InsertionMode::BeforeHead;
                        self.consume(token); // reconsume
                    },
                }
            }
            InsertionMode::BeforeHead => {
                match &token {
                    //                   tab       line feed    form feed     space
                    Token::Character('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}') => {} // Ignore the token.
                    Token::Comment(data) => {
                        // Insert a comment.
                        DOMNode::add_child(
                        self.current_node().clone(),
                        DOMNode::new(DOMNodeType::Comment {
                            data: data.to_string(),
                        })
                        .to_shared(),
                    )},
                    Token::DOCTYPE(_) => {} // parse error, ignore token
                    Token::Tag(tagdata) if tagdata.opening && tagdata.name == "html" => {
                        // Process the token using the rules for the "in body" insertion mode.
                        self.consume_in_mode(InsertionMode::InBody, token);
                    },
                    Token::Tag(tagdata) if tagdata.opening && tagdata.name == "head" => {
                        // Insert an HTML element for the token.
                        let head = DOMNode::new(DOMNodeType::Head).to_shared();
                        DOMNode::add_child(self.current_node().clone(), head.clone());
                        self.open_elements.push(head.clone());

                        // Set the head element pointer to the newly created head element.
                        self.head = Some(head);

                        // Switch the insertion mode to "in head".
                        self.insertion_mode = InsertionMode::InHead;
                    },
                    Token::Tag(tagdata)
                        if !tagdata.opening
                            && tagdata.name != "head"
                            && tagdata.name != "body"
                            && tagdata.name != "html"
                            && tagdata.name != "br" => {} // Parse error. Ignore the token.
                    _ => {
                        // Insert an HTML element for a "head" start tag token with no attributes.
                        let head = DOMNode::new(DOMNodeType::Head).to_shared();
                        DOMNode::add_child(self.current_node().clone(), head.clone());
                        self.open_elements.push(head.clone());

                        // Set the head element pointer to the newly created head element.
                        self.head = Some(head);
                        
                        // Switch the insertion mode to "in head".
                        self.insertion_mode = InsertionMode::InHead;

                        // Reprocess the current token.
                        self.consume(token);
                    },
                }
            }
            InsertionMode::InHead => {
                match token {
                    Token::Character(c @ ('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}')) => {
                        // Insert the character.
                        self.current_node().borrow().add_text(c.to_string());
                    }
                    Token::Comment(data) => {
                        // Insert a comment.
                        DOMNode::add_child(
                        self.current_node().clone(),
                        DOMNode::new(DOMNodeType::Comment { data: data }).to_shared(),
                    )},
                    Token::DOCTYPE(_) => {} // parse error, ignore token
                    Token::Tag(tagdata) if tagdata.opening && tagdata.name == "html"=> {
                        // Process the token using the rules for the "in body" insertion mode.
                        self.consume_in_mode(InsertionMode::InBody, Token::Tag(tagdata));
                    },
                    Token::Tag(tagdata) if tagdata.opening && (tagdata.name == "base" || tagdata.name == "basefont" || tagdata.name == "bgsound" || tagdata.name == "link") => {
                        // Insert an HTML element for the token. Immediately pop the
                        // current node off the stack of open elements.
                        //
                        // Acknowledge the token's self-closing flag, if it is set.
                        DOMNode::add_child(
                            self.current_node().clone(),
                            DOMNode::from(tagdata).to_shared(),
                        );
                    },
                    Token::Tag(tagdata) if tagdata.opening && tagdata.name == "meta" => {
                        // Insert an HTML element for the token. Immediately pop the current
                        // node off the stack of open elements.
                        //
                        // Acknowledge the token's self-closing flag, if it is set.
                        //
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
                        let meta_node = DOMNode::new(DOMNodeType::Meta).to_shared();
                        DOMNode::add_child(self.current_node().clone(), meta_node);
                    },
                    Token::Tag(tagdata) if tagdata.opening && tagdata.name == "title" => {
                        // Follow the generic RCDATA element parsing algorithm.
                        self.generic_rcdata_element_parsing_algorithm(tagdata);
                    },
                    Token::Tag(tagdata) if tagdata.opening && tagdata.name == "noscript" && self.scripting => {
                        // Follow the generic raw text element parsing algorithm.
                        self.generic_raw_text_element_parsing_algorithm(tagdata);
                    },
                    Token::Tag(tagdata) if tagdata.opening && (tagdata.name == "noframes" || tagdata.name == "style") => {
                        // Follow the generic raw text element parsing algorithm.
                        self.generic_raw_text_element_parsing_algorithm(tagdata);
                    },
                    Token::Tag(tagdata) if tagdata.opening && tagdata.name == "noscript" && !self.scripting => {
                        // Insert an HTML element for the token.
                        let domnode = DOMNode::new(DOMNodeType::NoScript).to_shared();
                        DOMNode::add_child(self.current_node().clone(), domnode);

                        // Switch the insertion mode to "in head noscript".
                        self.insertion_mode = InsertionMode::InHeadNoscript;
                    },
                    Token::Tag(tagdata) if tagdata.opening && tagdata.name == "script" => {
                        unimplemented!();
                    },
                    Token::Tag(tagdata) if !tagdata.opening && tagdata.name == "head" => {
                        // Pop the current node (which will be the head element) off the stack of open elements.
                        let top_node = self.open_elements.pop();
                        assert!(top_node.is_some());
                        assert_eq!(top_node.unwrap().borrow().node_type, DOMNodeType::Head);

                        // Switch the insertion mode to "after head".
                        self.insertion_mode = InsertionMode::AfterHead;
                    },
                    Token::Tag(tagdata) if tagdata.opening && tagdata.name == "template" => {
                        // Insert an HTML element for the token.
                        let domnode = DOMNode::new(DOMNodeType::Template).to_shared();
                        DOMNode::add_child(self.current_node().clone(), domnode);

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
                    Token::Tag(tagdata) if !tagdata.opening && tagdata.name == "template" => {
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
                    Token::Tag(tagdata) if (tagdata.opening && tagdata.name == "head") || (!tagdata.opening && tagdata.name != "body" && tagdata.name != "html" && tagdata.name != "br") => {}, // Parse error. Ignore the token.
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
            }
            InsertionMode::InHeadNoscript => {
                match token {
                    Token::DOCTYPE(_) => {} // Parse error. Ignore the token.
                    Token::Tag(tagdata) if tagdata.opening && tagdata.name == "html" => {
                        // Process the token using the rules for the "in body" insertion
                        // mode.
                        self.consume_in_mode(InsertionMode::InBody, Token::Tag(tagdata));
                    },
                    Token::Tag(tagdata) if !tagdata.opening && tagdata.name == "noscript" => {
                        // Pop the current node (which will be a noscript element) from
                        // the stack of open elements; the new current node will be a
                        // head element.
                        let top_node = self.open_elements.pop();
                        assert!(top_node.is_some());
                        assert_eq!(
                            top_node.unwrap().borrow().node_type,
                            DOMNodeType::NoScript
                        );
                        assert!(
                            self.current_node().borrow().node_type == DOMNodeType::Head
                        );

                        // Switch the insertion mode to "in head".
                        self.insertion_mode = InsertionMode::InHead;
                    },
                    Token::Character('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}')
                        | Token::Comment(_) => {
                        // Process the token using the rules for the "in head" insertion
                        // mode.
                        self.consume_in_mode(InsertionMode::InHead, token);
                    },
                    Token::Tag(tagdata) if tagdata.opening && (tagdata.name == "basefont" || tagdata.name == "bgsound" || tagdata.name == "link" || tagdata.name == "meta" || tagdata.name == "style") => {
                        // Process the token using the rules for the "in head" insertion
                        // mode.
                        self.consume_in_mode(InsertionMode::InHead, Token::Tag(tagdata));
                    }
                    Token::Tag(tagdata) if tagdata.opening && (tagdata.name == "head" || tagdata.name == "noscript") => {
                    }, // Parse error. Ignore the token.
                    Token::Tag(tagdata) if !tagdata.opening && (tagdata.name != "br") => {}, // Parse error. Ignore the token.
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
            }
            InsertionMode::AfterHead => {
                match token {
                    //                        tab       line feed    form feed     space
                    Token::Character(c @ ('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}')) => {
                        // Insert the character.
                        self.current_node().borrow_mut().add_text(c.to_string());
                    },
                    Token::Comment(data) => {
                        // Insert a comment.
                        DOMNode::add_child(
                            self.current_node().clone(),
                            DOMNode::new(DOMNodeType::Comment { data: data }).to_shared(),
                        );
                    }
                    Token::DOCTYPE(_) => {} // Parse error. Ignore the token.
                    _ => todo!(),
                }
            }
            _ => todo!(),
        }
    }

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

    fn generic_raw_text_element_parsing_algorithm(&mut self, tagdata: TagData) {
        // Insert an HTML element for the token.
        let domnode = DOMNode::from(tagdata).to_shared();
        DOMNode::add_child(self.current_node().clone(), domnode);

        // switch the tokenizer to the RCDATA state.
        self.tokenizer.switch_to(TokenizerState::RAWTEXTState);

        // Let the original insertion mode be the current insertion mode.
        self.original_insertion_mode = Some(self.insertion_mode);

        // Then, switch the insertion mode to "text".
        self.insertion_mode = InsertionMode::Text;
    }
}

