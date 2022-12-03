use crate::dom::{DOMNode, DOMNodeType, SharedDOMNode};
use crate::tokenizer::{TagData, Token};

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

pub struct Parser {
    insertion_mode: InsertionMode,
    open_elements: Vec<SharedDOMNode>,
    head: Option<SharedDOMNode>,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            insertion_mode: InsertionMode::Initial,
            open_elements: vec![DOMNode::new(DOMNodeType::Document).to_shared()],
            head: None,
        }
    }

    fn current_node(&self) -> &SharedDOMNode {
        // safe because self.open_elements can never be empty
        self.open_elements.last().unwrap()
    }

    pub fn consume(&mut self, token: Token) {
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
                    Token::Tag(tagdata @ TagData { opening: false, .. }) => {
                        if tagdata.name == "head"
                            || tagdata.name == "body"
                            || tagdata.name == "html"
                            || tagdata.name == "br"
                        {
                            // same as anything else
                            self.before_html_anything_else(token);
                        } else {
                            // parse error, ignore
                        }
                    }
                    Token::Tag(tagdata @ TagData { opening: true, .. }) => {
                        if tagdata.name == "html" {
                            let dom_element = DOMNode::new(DOMNodeType::Html).to_shared();
                            DOMNode::add_child(self.current_node().clone(), dom_element.clone());
                            self.open_elements.push(dom_element);
                            self.insertion_mode = InsertionMode::BeforeHead;
                        } else {
                            // same as anything else
                            self.before_html_anything_else(token);
                        }
                    }
                    _ => self.before_html_anything_else(token),
                }
            }
            InsertionMode::BeforeHead => {
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
                    Token::Tag(tagdata @ TagData { opening: true, .. }) => {
                        if tagdata.name == "html" {
                            self.consume_in_mode(InsertionMode::InBody, token);
                        } else if tagdata.name == "head" {
                            let head = DOMNode::new(DOMNodeType::Head).to_shared();
                            self.head = Some(head.clone());
                            DOMNode::add_child(self.current_node().clone(), head.clone());
                            self.open_elements.push(head);
                            self.insertion_mode = InsertionMode::InHead;
                        } else {
                            self.before_head_anything_else(token);
                        }
                    }
                    Token::Tag(tagdata @ TagData { opening: false, .. }) => {
                        if tagdata.name == "head"
                            || tagdata.name == "body"
                            || tagdata.name == "html"
                            || tagdata.name == "br"
                        {
                            self.before_head_anything_else(token);
                        } else {
                            // parse error, ignore token
                        }
                    }
                    _ => self.before_head_anything_else(token),
                }
            }
            InsertionMode::InHead => match token {
                Token::Character(c @ ('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}')) => {
                    self.current_node().borrow().add_text(c.to_string());
                }
                _ => unimplemented!(),
            },
            _ => todo!(),
        }
    }

    fn before_html_anything_else(&mut self, token: Token) {
        let dom_element = DOMNode::new(DOMNodeType::Html).to_shared();
        DOMNode::add_child(self.current_node().clone(), dom_element.clone());
        self.open_elements.push(dom_element);
        self.insertion_mode = InsertionMode::BeforeHead;
        self.consume(token); // reconsume
    }

    fn before_head_anything_else(&mut self, token: Token) {
        let head = DOMNode::new(DOMNodeType::Head).to_shared();
        self.head = Some(head.clone());
        DOMNode::add_child(self.current_node().clone(), head.clone());
        self.open_elements.push(head);
        self.insertion_mode = InsertionMode::InHead;
        self.consume(token); // reconsume
    }
}
