//! Implements the [Tree Construction Stage](https://html.spec.whatwg.org/multipage/parsing.html#tree-construction)

use std::mem;

use crate::{
    css::{self, Stylesheet},
    dom::{
        self,
        dom_objects::{
            Comment, Document, DocumentType, Element, HtmlBodyElement, HtmlDdElement,
            HtmlDivElement, HtmlElement, HtmlFormElement, HtmlHeadElement, HtmlHtmlElement,
            HtmlLiElement, HtmlLinkElement, HtmlParagraphElement, HtmlScriptElement,
            HtmlTableElement, HtmlTemplateElement, Node, Text,
        },
        DomPtr, DomType, DomTyped,
    },
    html::{
        links,
        tokenization::{ParseErrorHandler, TagData, Token, Tokenizer, TokenizerState},
        treebuilding::{ActiveFormattingElement, ActiveFormattingElements, FormatEntry},
    },
    infra::Namespace,
    static_interned, InternedString,
};

use sl_std::iter::IteratorExtensions;

const TAB: char = '\u{0009}';
const LINE_FEED: char = '\u{000A}';
const FORM_FEED: char = '\u{000C}';
const WHITESPACE: char = '\u{0020}';

// FIXME: We should also consider the object namespaces here (and in every other scope)
/// <https://html.spec.whatwg.org/multipage/parsing.html#has-an-element-in-scope>
const DEFAULT_SCOPE: &[InternedString] = &[
    static_interned!("applet"),
    static_interned!("caption"),
    static_interned!("html"),
    static_interned!("table"),
    static_interned!("td"),
    static_interned!("th"),
    static_interned!("marquee"),
    static_interned!("object"),
    static_interned!("template"),
    static_interned!("mi"),
    static_interned!("mi"),
    static_interned!("mn"),
    static_interned!("ms"),
    static_interned!("mtext"),
    static_interned!("annotation-xml"),
    static_interned!("foreignObject"),
    static_interned!("desc"),
    static_interned!("title"),
];

/// <https://html.spec.whatwg.org/multipage/parsing.html#has-an-element-in-button-scope>
const BUTTON_SCOPE: &[InternedString] = &[
    static_interned!("html"),
    static_interned!("template"),
    static_interned!("button"),
];

/// <https://html.spec.whatwg.org/multipage/parsing.html#has-an-element-in-list-item-scope>
const LIST_ITEM_SCOPE: &[InternedString] = &[
    static_interned!("html"),
    static_interned!("template"),
    static_interned!("ol"),
    static_interned!("ul"),
];

/// <https://html.spec.whatwg.org/multipage/parsing.html#has-an-element-in-table-scope>
const TABLE_SCOPE: &[InternedString] = &[
    static_interned!("html"),
    static_interned!("template"),
    static_interned!("table"),
];

#[derive(Clone, Copy, Debug)]
enum GenericParsingAlgorithm {
    RcData,
    RawText,
}

/// <https://html.spec.whatwg.org/multipage/parsing.html#parse-state>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FramesetOkFlag {
    #[default]
    Ok,
    NotOk,
}

pub struct Parser<P: ParseErrorHandler> {
    tokenizer: Tokenizer<P>,
    document: DomPtr<Document>,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#original-insertion-mode>
    original_insertion_mode: Option<InsertionMode>,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#stack-of-template-insertion-modes>
    template_insertion_modes: Vec<InsertionMode>,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#insertion-mode>
    insertion_mode: InsertionMode,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#stack-of-open-elements>
    open_elements: Vec<DomPtr<Element>>,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#head-element-pointer>
    head: Option<DomPtr<HtmlHeadElement>>,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#frameset-ok-flag>
    frameset_ok: FramesetOkFlag,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#form-element-pointer>
    form: Option<DomPtr<HtmlFormElement>>,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#list-of-active-formatting-elements>
    active_formatting_elements: ActiveFormattingElements,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#scripting-flag>
    execute_script: bool,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#concept-pending-table-char-tokens>
    pending_table_character_tokens: Vec<char>,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#foster-parent>
    is_foster_parenting_enabled: bool,

    done: bool,

    stylesheets: Vec<Stylesheet>,
}

impl<P: ParseErrorHandler> Parser<P> {
    pub fn new(source: &str) -> Self {
        let document = DomPtr::new(Document::default());
        // TODO: judging from the spec behaviour, it appears that document's document's
        // point to themselves. We should find a note for that somewhere in a spec.
        document
            .borrow_mut()
            .set_owning_document(DomPtr::clone(&document).downgrade());

        Self {
            tokenizer: Tokenizer::new(source),
            document,
            original_insertion_mode: None,
            template_insertion_modes: vec![],
            insertion_mode: InsertionMode::Initial,
            open_elements: vec![],
            head: None,
            form: None,
            frameset_ok: FramesetOkFlag::default(),
            active_formatting_elements: ActiveFormattingElements::default(),
            execute_script: false,
            pending_table_character_tokens: vec![],
            is_foster_parenting_enabled: false,
            done: false,
            stylesheets: vec![Stylesheet::user_agent_rules()],
        }
    }

    #[must_use]
    fn open_elements_bottommost_node(&self) -> Option<DomPtr<Element>> {
        self.open_elements.last().cloned()
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#acknowledge-self-closing-flag>
    #[inline]
    fn acknowledge_self_closing_flag_if_set(&self, _tag: &TagData) {
        // This is only relevant for detecting parse errors, which we currently don't care about
    }

    #[must_use]
    fn find_in_open_elements<T: DomTyped>(&self, needle: &DomPtr<T>) -> Option<usize> {
        self.open_elements
            .iter()
            .enumerate()
            .find(|(_, node)| node.ptr_eq(needle))
            .map(|(i, _)| i)
    }

    /// Pop elements from the stack of open elements until a element matching a predicate has been popped
    fn pop_from_open_elements_until<F: Fn(DomPtr<Element>) -> bool>(&mut self, predicate: F) {
        loop {
            if predicate(self.pop_from_open_elements()) {
                break;
            }
        }
    }

    fn remove_from_open_elements<T: DomTyped>(&mut self, to_remove: &DomPtr<T>) {
        self.open_elements
            .retain_mut(|element| !DomPtr::ptr_eq(to_remove, element))
    }

    fn pop_from_open_elements(&mut self) -> DomPtr<Element> {
        let element = self
            .open_elements
            .pop()
            .expect("there are no open elements to pop");

        // FIXME: Clean up the way we check for new stylesheets here (<style>, <link rel="stylesheet">)

        // Check if we just popped a <style> element, if so, register a new stylesheet
        if element.underlying_type() == DomType::HtmlStyleElement {
            if let Some(first_child) = element.borrow().children().first() {
                if let Some(text_node) = first_child.try_into_type::<Text>() {
                    if let Ok(stylesheet) =
                        css::Parser::new(text_node.borrow().content(), css::Origin::Author)
                            .parse_stylesheet(self.stylesheets.len())
                    {
                        if !stylesheet.rules().is_empty() {
                            self.stylesheets.push(stylesheet);
                        } else {
                            log::debug!("Dropping empty stylesheet");
                        }
                    }
                }
            }
        }

        if let Some(link_element) = element.try_into_type::<HtmlLinkElement>() {
            let link_element = link_element.borrow();
            if link_element.relationship() == links::Relationship::Stylesheet {
                if let Some(url) = link_element.url() {
                    match mime::Resource::load(&url) {
                        Ok(resource) => {
                            // FIXME: Check mime type here
                            let css = String::from_utf8_lossy(&resource.data);
                            if let Ok(stylesheet) = css::Parser::new(&css, css::Origin::Author)
                                .parse_stylesheet(self.stylesheets.len())
                            {
                                if !stylesheet.rules().is_empty() {
                                    self.stylesheets.push(stylesheet);
                                } else {
                                    log::debug!("Dropping empty stylesheet");
                                }
                            }
                        },
                        Err(error) => {
                            log::error!(
                                "Failed to load stylesheet: {url} could not be loaded ({error:?}"
                            )
                        },
                    }
                }
            }
        }

        element
    }

    pub fn parse(mut self) -> (DomPtr<Document>, Vec<Stylesheet>) {
        while let Some(token) = self.tokenizer.next() {
            self.consume(token);

            if self.done {
                break;
            }
        }

        (self.document, self.stylesheets)
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#current-node>
    fn current_node(&self) -> DomPtr<Element> {
        // The current node is the bottommost node in this stack of open elements.
        self.open_elements_bottommost_node()
            .expect("Stack of open elements is empty")
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#current-template-insertion-mode>
    #[inline]
    #[must_use]
    fn current_template_insertion_mode(&self) -> InsertionMode {
        *self
            .template_insertion_modes
            .last()
            .expect("Stack of template insertion modes cannot be empty")
    }

    fn consume(&mut self, token: Token) {
        self.consume_in_mode(self.insertion_mode, token);
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#insert-a-character>
    pub fn insert_character(&self, c: char) {
        // 1. Let data be the characters passed to the algorithm, or, if no characters were explicitly specified,
        // the character of the character token being processed.
        // NOTE: We always pass the character to the algorithm.

        // 2. Let the adjusted insertion location be the appropriate place for inserting a node.
        let adjusted_insert_location = self.appropriate_place_for_inserting_node();

        // 3. If the adjusted insertion location is in a Document node
        if adjusted_insert_location.is_a::<Document>() {
            // then return.
            return;
        }

        // 4. If there is a Text node immediately before the adjusted insertion location
        if let Some(last_child) = adjusted_insert_location.borrow().last_child() {
            if let Some(text) = last_child.try_into_type::<Text>() {
                // then append data to that Text node's data.
                text.borrow_mut().content_mut().push(c);
                return;
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
        let new_node = DomPtr::new(new_text).upcast();
        Node::append_child(adjusted_insert_location, new_node)
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#appropriate-place-for-inserting-a-node>
    fn appropriate_place_for_inserting_node(&self) -> DomPtr<Node> {
        self.appropriate_place_for_inserting_node_with_override(None)
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#appropriate-place-for-inserting-a-node>
    fn appropriate_place_for_inserting_node_with_override(
        &self,
        override_target: Option<DomPtr<Node>>,
    ) -> DomPtr<Node> {
        // If there was an override target specified, then let target be the override target.
        // Otherwise, let target be the current node.
        let target = override_target.unwrap_or_else(|| self.current_node().upcast());

        let adjusted_insertion_location = if self.is_foster_parenting_enabled {
            // If foster parenting is enabled and target is a table, tbody, tfoot, thead, or tr element
            log::warn!("FIXME: implement foster parenting in appropriate_place_for_inserting_node");
            target
        } else {
            // Let adjusted insertion location be inside target, after its last child (if any).
            target
        };

        // TODO:
        // If the adjusted insertion location is inside a template element, let it instead be
        // inside the template element's template contents, after its last child (if any).

        // Return the adjusted insertion location.
        adjusted_insertion_location
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#insert-a-comment>
    fn insert_comment_at(&mut self, data: String, position: Option<DomPtr<Node>>) {
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
        let new_node = DomPtr::new(new_comment).upcast();
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
            GenericParsingAlgorithm::RawText => self.tokenizer.switch_to(TokenizerState::RAWTEXT),
            GenericParsingAlgorithm::RcData => self.tokenizer.switch_to(TokenizerState::RCDATA),
        }

        // Let the original insertion mode be the current insertion mode.
        self.original_insertion_mode = Some(self.insertion_mode);

        // Then, switch the insertion mode to "text".
        self.insertion_mode = InsertionMode::Text;
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#has-an-element-in-scope>
    fn is_element_in_scope(&self, element_name: InternedString) -> bool {
        // FIXME: this default scope should contain more types but they dont exist yet
        self.is_element_in_specific_scope(element_name, DEFAULT_SCOPE)
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#has-an-element-in-button-scope>
    fn is_element_in_button_scope(&self, element_name: InternedString) -> bool {
        self.is_element_in_specific_scope(element_name, BUTTON_SCOPE)
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#has-an-element-in-list-item-scope>
    fn is_element_in_list_item_scope(&self, element_name: InternedString) -> bool {
        self.is_element_in_specific_scope(element_name, LIST_ITEM_SCOPE)
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#has-an-element-in-table-scope>
    fn is_element_in_table_scope(&self, element_name: InternedString) -> bool {
        self.is_element_in_specific_scope(element_name, TABLE_SCOPE)
    }

    fn elements_in_scope<'open_elements, 'scope, 'iterator>(
        &'open_elements self,
        scope: &'scope [InternedString],
    ) -> impl 'iterator + Iterator<Item = DomPtr<Element>>
    where
        'open_elements: 'iterator,
        'scope: 'iterator,
    {
        self.open_elements
            .iter()
            .rev()
            .take_while_including(|e| !scope.contains(&e.borrow().local_name()))
            .cloned()
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#has-an-element-in-the-specific-scope>
    fn is_element_in_specific_scope(
        &self,
        element_name: InternedString,
        scope: &[InternedString],
    ) -> bool {
        // NOTE: elements_in_scope takes care of 1. and 3. for us

        // 1. Initialize node to be the current node (the bottommost node of the stack).
        for node in self.elements_in_scope(scope) {
            let local_name = node.borrow().local_name();

            // 2. If node is the target node, terminate in a match state.
            if local_name == element_name {
                return true;
            }

            // 3. Otherwise, if node is one of the element types in list, terminate in a failure state.

            // 4. Otherwise, set node to the previous entry in the stack of open elements and return to step 2.
            //    (This will never fail, since the loop will always terminate in the previous step if the
            //    top of the stack — an html element — is reached.)
        }
        false
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#close-a-p-element>
    fn close_p_element(&mut self) {
        // 1. Generate implied end tags, except for p elements.
        self.generate_implied_end_tags_excluding(Some(static_interned!("p")));

        // 2. If the current node is not a p element,
        if !self.current_node().is_a::<HtmlParagraphElement>() {
            // then this is a parse error.
        }

        // 3. Pop elements from the stack of open elements until a p element has been popped from the stack.
        self.pop_from_open_elements_until(|node| node.is_a::<HtmlParagraphElement>());
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#closing-elements-that-have-implied-end-tags>
    #[inline]
    fn generate_implied_end_tags(&mut self) {
        self.generate_implied_end_tags_excluding(None);
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#closing-elements-that-have-implied-end-tags>
    fn generate_implied_end_tags_excluding(&mut self, exclude: Option<InternedString>) {
        const LOOP_WHILE_ELEMENTS: [InternedString; 10] = [
            static_interned!("dd"),
            static_interned!("dt"),
            static_interned!("li"),
            static_interned!("optgroup"),
            static_interned!("option"),
            static_interned!("p"),
            static_interned!("rb"),
            static_interned!("rp"),
            static_interned!("rt"),
            static_interned!("rtc"),
        ];

        loop {
            let element_name = self.current_node().borrow().local_name();

            if !LOOP_WHILE_ELEMENTS.contains(&element_name)
                || exclude.is_some_and(|e| e == element_name)
            {
                break;
            }

            self.pop_from_open_elements();
        }
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#generate-all-implied-end-tags-thoroughly>
    fn generate_implied_end_tags_thoroughly(&mut self) {
        loop {
            let current_node = self.current_node();
            // FIXME: There are more elements here that aren't yet implemented
            if current_node.is_a::<HtmlParagraphElement>() {
                return;
            }
            self.pop_from_open_elements();
        }
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#stop-parsing>
    fn stop_parsing(&mut self) {
        self.done = true;
        // FIXME: this is ad-hoc, we don't support most of what is necessary here

        // FIXME: I assume this must be done at some point, but i can't find it in the spec
        let html_element = self
            .open_elements
            .first()
            .expect("no root element found")
            .clone()
            .upcast();
        Node::append_child(self.document.clone().upcast(), html_element);

        // 4. Pop all the nodes off the stack of open elements.
        while !self.open_elements.is_empty() {
            self.pop_from_open_elements();
        }
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#reset-the-insertion-mode-appropriately>
    fn reset_insertion_mode_appropriately(&mut self) {
        // 1. Let last be false.
        let mut last = false;

        // 2. Let node be the last node in the stack of open elements.
        let mut node = self.current_node();
        let mut node_index = self.open_elements.len() - 1;

        // 3. Loop: If node is the first node in the stack of open elements, then set last to true, and,
        //    if the parser was created as part of the HTML fragment parsing algorithm (fragment case),
        //    set node to the context element passed to that algorithm.
        loop {
            if node_index == 0 {
                last = true;
            }

            // 4. FIXME: If node is a select element, run these substeps:

            // 5. FIXME: If node is a td or th element and last is false, then switch the insertion mode to "in cell" and return.

            // 6. FIXME: If node is a tr element, then switch the insertion mode to "in row" and return.

            // 7. FIXME: If node is a tbody, thead, or tfoot element, then switch the insertion mode to "in table body" and return.

            // 8. FIXME: If node is a caption element, then switch the insertion mode to "in caption" and return.

            // 9. FIXME: If node is a colgroup element, then switch the insertion mode to "in column group" and return.

            // 10. FIXME: If node is a table element, then switch the insertion mode to "in table" and return.

            // 11: If node is a template element, then switch the insertion mode to the current template insertion mode and return.
            if node.is_a::<HtmlTemplateElement>() {
                self.insertion_mode = self.current_template_insertion_mode();
                return;
            }

            // 12. If node is a head element and last is false, then switch the insertion mode to "in head" and return.
            if node.is_a::<HtmlHeadElement>() {
                self.insertion_mode = InsertionMode::InHead;
                return;
            }

            // 13. If node is a body element, then switch the insertion mode to "in body" and return.
            if node.is_a::<HtmlBodyElement>() {
                self.insertion_mode = InsertionMode::InBody;
                return;
            }

            // 14. FIXME: If node is a frameset element, then switch the insertion mode to "in frameset" and return. (fragment case)

            // 15. If node is an html element, run these substeps:
            if node.is_a::<HtmlHtmlElement>() {
                // 1. If the head element pointer is null, switch the insertion mode to "before head" and return. (fragment case)
                if self.head.is_none() {
                    self.insertion_mode = InsertionMode::BeforeHead;
                    return;
                }
                // 2. Otherwise, the head element pointer is not null, switch the insertion mode to "after head" and return.
                else {
                    self.insertion_mode = InsertionMode::AfterHead;
                    return;
                }
            }

            // 16. If last is true, then switch the insertion mode to "in body" and return. (fragment case)
            if last {
                self.insertion_mode = InsertionMode::InBody;
                return;
            }

            // 17. Let node now be the node before node in the stack of open elements.
            node_index -= 1;
            node = self.open_elements[node_index].clone();

            // 18. Return to the step labeled loop.
        }
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#create-an-element-for-the-token>
    fn create_element_for_token(
        &self,
        tagdata: &TagData,
        namespace: Namespace,
        intended_parent: &DomPtr<Node>,
    ) -> DomPtr<Element> {
        // FIXME: If the active speculative HTML parser is not null, then return the result of creating a speculative mock element
        // given given namespace, the tag name of the given token, and the attributes of the given token.

        // FIXME: Otherwise, optionally create a speculative mock element given given namespace, the tag name of the given token, and the attributes of the given token.

        // Let document be intended parent's node document.
        let document = intended_parent
            .borrow()
            .owning_document()
            .expect("Intended Parent does not belong to a document");

        // Let local name be the tag name of the token.
        let local_name = tagdata.name;

        // Let is be the value of the "is" attribute in the given token, if such an attribute exists, or null otherwise.
        let is = tagdata.lookup_attribute(static_interned!("is"));

        // Let definition be the result of looking up a custom element definition given document, given namespace, local name, and is.
        let _definition = dom::lookup_custom_element_definition(namespace, local_name, is);

        // FIXME: If definition is non-null and the parser was not created as part of the HTML fragment parsing algorithm, then let will execute script be true. Otherwise, let it be false.

        // FIXME: If will execute script is true, then:
        //      Increment document's throw-on-dynamic-markup-insertion counter.
        //      If the JavaScript execution context stack is empty, then perform a microtask checkpoint.
        //      Push a new element queue onto document's relevant agent's custom element reactions stack.

        // Let element be the result of creating an element given document, localName, given namespace, null, and is.
        // If will execute script is true, set the synchronous custom elements flag; otherwise, leave it unset.
        let element = dom::create_element(
            document.downgrade(),
            local_name,
            namespace,
            None,
            is.map(|is| is.to_owned()),
            self.execute_script,
        );

        // Append each attribute in the given token to element.
        for (key, value) in tagdata.attributes() {
            element.borrow_mut().append_attribute(*key, *value);
        }

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

    /// <https://html.spec.whatwg.org/multipage/parsing.html#insert-an-html-element>
    fn insert_html_element_for_token(&mut self, tagdata: &TagData) -> DomPtr<Element> {
        self.insert_foreign_element(tagdata, Namespace::HTML, false)
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#insert-a-foreign-element>
    fn insert_foreign_element(
        &mut self,
        tagdata: &TagData,
        namespace: Namespace,
        only_add_to_element_stack: bool,
    ) -> DomPtr<Element> {
        // 1. Let the adjusted insertion location be the appropriate place for inserting a node.
        let adjusted_insertion_location = self.appropriate_place_for_inserting_node();

        // 2. Let element be the result of creating an element for the token in the given namespace,
        //    with the intended parent being the element in which the adjusted insertion location finds itself.
        let element =
            self.create_element_for_token(tagdata, namespace, &adjusted_insertion_location);

        // 3. If onlyAddToElementStack is false, then run insert an element at the adjusted insertion location with element.
        if !only_add_to_element_stack {
            Node::append_child(adjusted_insertion_location, element.clone().upcast());
        }

        // 4. Push element onto the stack of open elements so that it is the new current node.
        self.open_elements.push(element.clone());

        // 5. Return element.
        element
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#clear-the-stack-back-to-a-table-context>
    fn clear_the_stack_back_to_a_table_context(&mut self) {
        while !TABLE_SCOPE.contains(&self.current_node().borrow().local_name()) {
            self.open_elements.pop();
        }
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#reconstruct-the-active-formatting-elements>
    fn reconstruct_active_formatting_elements(&mut self) {
        let is_marker_or_in_open_elements = |entry: &FormatEntry| match entry {
            FormatEntry::Element(format_element)
                if !self
                    .open_elements
                    .iter()
                    .any(|e| DomPtr::ptr_eq(e, &format_element.element)) =>
            {
                false
            },
            FormatEntry::Marker | FormatEntry::Element(_) => true,
        };

        match self.active_formatting_elements.last() {
            None => {
                // 1. If there are no entries in the list of active formatting elements,
                //    then there is nothing to reconstruct; stop this algorithm.
                return;
            },
            Some(entry) => {
                // 2. If the last (most recently added) entry in the list of active formatting es is a marker,
                //    or if it is an element that is in the stack of open elements, then there is nothing to reconstruct;
                //    stop this algorithm.
                if is_marker_or_in_open_elements(entry) {
                    return;
                }
            },
        }

        // 3. Let entry be the last (most recently added) element in the list of active formatting elements.
        let mut entry_index = self.active_formatting_elements.elements().len();

        loop {
            // 4. Rewind: If there are no entries before entry in the list of active formatting elements, then jump to the step labeled create.
            if entry_index == 0 {
                break;
            }

            // 5. Let entry be the entry one earlier than entry in the list of active formatting elements.
            entry_index -= 1;

            // 6. If entry is neither a marker nor an element that is also in the stack of open elements, go to the step labeled rewind.
            if !is_marker_or_in_open_elements(
                &self.active_formatting_elements.elements()[entry_index],
            ) {
                continue;
            }

            // 7. Advance: Let entry be the element one later than entry in the list of active formatting elements.
            entry_index += 1;
            break;
        }

        loop {
            let tag = match self.active_formatting_elements.elements_mut()[entry_index] {
                FormatEntry::Marker => panic!("cannot be a marker element"),
                FormatEntry::Element(ref mut formatting_element) => formatting_element.tag.clone(),
            };

            // 8. Create: Insert an HTML element for the token for which the element entry was created, to obtain new element.
            let new_element = self.insert_html_element_for_token(&tag);

            // 9. Replace the entry for entry in the list with an entry for new element.
            self.active_formatting_elements.elements_mut()[entry_index] =
                FormatEntry::Element(ActiveFormattingElement {
                    element: new_element,
                    tag,
                });

            // 10. If the entry for new element in the list of active formatting elements is not the last entry in the list, return to the step labeled advance.
            // NOTE: Because "Advance" is so simple, we just replicate the step here to avoid making the
            //       control flow even more confusing than it already is
            entry_index += 1;
        }
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#adoption-agency-algorithm>
    fn run_adoption_agency_algorithm(&mut self, tagdata: &TagData) {
        // 1. Let subject be token's tag name.
        let subject = tagdata.name;

        // 2. If the current node is an HTML element whose tag name is subject, and the current node is not
        //    in the list of active formatting elements, then pop the current node off the stack of open elements and return.
        let current_node = self.current_node();
        if let Some(html_element) = current_node.try_into_type::<HtmlElement>() {
            if html_element.borrow().local_name() == subject
                && !self
                    .active_formatting_elements
                    .list()
                    .any(|formatting_element| {
                        DomPtr::ptr_eq(&formatting_element.element, &current_node)
                    })
            {
                self.pop_from_open_elements();
                return;
            }
        }

        // 3. Let outer loop counter be 0.
        let mut outer_loop_counter = 0;

        // 4. While true:
        loop {
            // 1. If outer loop counter is greater than or equal to 8, then return.
            if outer_loop_counter >= 8 {
                return;
            }

            // 2. Increment outer loop counter by 1.
            outer_loop_counter += 1;

            // 3. Let formatting element be the last element in the list of active formatting elements that:
            //      * is between the end of the list and the last marker in the list, if any, or the start of the list otherwise, and
            //      * has the tag name subject.
            //    If there is no such element, then return and instead act as described in the "any other end tag" entry above.
            let last_such_element = self
                .active_formatting_elements
                .elements_since_last_marker()
                .enumerate()
                .find(|(_, format_element)| {
                    format_element.element.borrow().local_name() == subject
                });
            let (index_relative_to_last_marker, formatting_element) = match last_such_element {
                Some((index, formatting_element)) => (index, formatting_element.clone()),
                None => {
                    self.any_other_end_tag_in_body(tagdata.to_owned());
                    return;
                },
            };

            // 4. If formatting element is not in the stack of open elements,
            //    then this is a parse error; remove the element from the list, and return.
            // NOTE: we need the index later, this is why this is more complicated than it needs to be.
            let index_in_open_elements = match self
                .find_in_open_elements(&formatting_element.element)
            {
                Some(i) => i,
                None => {
                    self.active_formatting_elements
                        .remove_element_at_index_from_last_marker(index_relative_to_last_marker);
                    return;
                },
            };

            // 5. If formatting element is in the stack of open elements, but the element is not in scope,
            //    then this is a parse error; return.
            if !self.is_element_in_scope(formatting_element.element.borrow().local_name()) {
                return;
            }

            // 6. If formatting element is not the current node, this is a parse error. (But do not return.)

            // 7. Let furthest block be the topmost node in the stack of open elements that is lower in the stack than formatting element,
            //    and is an element in the special category. There might not be one.
            let furthest_block = self.open_elements[index_in_open_elements..]
                .iter()
                .enumerate()
                .rev()
                .filter_map(|(i, node)| Some((i, node.try_into_type::<Element>()?)))
                .find(|(_, element)| is_element_in_special_category(element.borrow().local_name()));

            match furthest_block {
                None => {
                    // 8. If there is no furthest block, then the UA must first pop all the nodes from the bottom of the stack of open elements,
                    //    from the current node up to and including formatting element, then remove formatting element from the
                    //    list of active formatting elements, and finally return.
                    while self.open_elements.len() != index_in_open_elements {
                        self.pop_from_open_elements();
                    }

                    self.active_formatting_elements
                        .remove_element_at_index_from_last_marker(index_relative_to_last_marker);

                    return;
                },
                Some((furthest_block_index, furthest_block)) => {
                    // 9. Let common ancestor be the element immediately above formatting element in the stack of open elements.
                    let common_ancestor = formatting_element
                        .element
                        .borrow()
                        .parent_node()
                        .expect("Common ancestor cannot be None");

                    // 10. Let a bookmark note the position of formatting element in the list of active formatting elements
                    //     relative to the elements on either side of it in the list.

                    // 11. Let node and last node be furthest block.
                    let mut node;
                    let mut node_index = furthest_block_index;
                    let mut last_node = furthest_block.clone();

                    // 12. Let inner loop counter be 0.
                    let mut inner_loop_counter = 0;

                    // 13. While true:
                    loop {
                        // 1. Increment inner loop counter by 1.
                        inner_loop_counter += 1;

                        // 2. Let node be the element immediately above node in the stack of open elements,
                        //    or if node is no longer in the stack of open elements (e.g. because it got removed by this algorithm),
                        //    the element that was immediately above node in the stack of open elements before node was removed.
                        node_index -= 1;
                        node = self.open_elements[node_index].clone();

                        // 3. If node is formatting element, then break.
                        if DomPtr::ptr_eq(&node, &formatting_element.element) {
                            break;
                        }

                        // 4. If inner loop counter is greater than 3 and node is in the list of active formatting elements,
                        //    then remove node from the list of active formatting elements.
                        if inner_loop_counter > 3 {
                            self.active_formatting_elements.remove(&node);
                        }

                        // 5. If node is not in the list of active formatting elements,
                        //    then remove node from the stack of open elements and continue.
                        let node_position_in_formatting_elements =
                            match self.active_formatting_elements.find(&node) {
                                Some(index) => index,
                                None => {
                                    self.remove_from_open_elements(&node);
                                    continue;
                                },
                            };

                        // 6. Create an element for the token for which the element node was created, in the HTML namespace,
                        //    with common ancestor as the intended parent; replace the entry for node in the
                        //    list of active formatting elements with an entry for the new element,
                        //    replace the entry for node in the stack of open elements with an entry for the new element,
                        //    and let node be the new element.
                        let tag = match &self.active_formatting_elements.elements()
                            [node_position_in_formatting_elements]
                        {
                            FormatEntry::Element(format_element) => format_element.tag.clone(),
                            FormatEntry::Marker => unreachable!("entry cannot be a marker"),
                        };
                        let new_element =
                            self.create_element_for_token(&tag, Namespace::HTML, &common_ancestor);
                        self.open_elements[node_index] = new_element.clone();
                        node = new_element;

                        // 7. FIXME: If last node is furthest block, then move the aforementioned bookmark to be immediately after the
                        //    new node in the list of active formatting elements.

                        // 8. Append last node to node.
                        Node::append_child(node.clone().upcast(), last_node.upcast());

                        // 9. Set last node to node.
                        last_node = node;
                    }

                    // 14. Insert whatever last node ended up being in the previous step at the appropriate place for inserting a node,
                    //     but using common ancestor as the override target.
                    let appropriate_place = self
                        .appropriate_place_for_inserting_node_with_override(Some(common_ancestor));
                    Node::append_child(appropriate_place, last_node.upcast());

                    // 15. Create an element for the token for which formatting element was created,
                    //     in the HTML namespace, with furthest block as the intended parent.
                    let new_element = self.create_element_for_token(
                        &formatting_element.tag,
                        Namespace::HTML,
                        &furthest_block.clone().upcast(),
                    );

                    // 16. Take all of the child nodes of furthest block and append them to the element created in the last step.
                    for child in furthest_block.borrow().children() {
                        Node::append_child(new_element.clone().upcast(), child.clone());
                    }

                    // 17. Append that new element to furthest block.
                    Node::append_child(furthest_block.upcast(), new_element.upcast());

                    // 18. FIXME: Remove formatting element from the list of active formatting elements,
                    //     and insert the new element into the list of active formatting elements
                    //     at the position of the aforementioned bookmark.

                    // 19. FIXME: Remove formatting element from the stack of open elements,
                    //     and insert the new element into the stack of open elements immediately below the position
                    //     of furthest block in that stack.
                },
            }
        }
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inhtml>
    fn consume_in_mode(&mut self, mode: InsertionMode, token: Token) {
        log::trace!(
            "Consuming {token:?} in {mode:?}.\nThe current token is a {:?}",
            self.open_elements_bottommost_node()
                .as_ref()
                .map(DomPtr::underlying_type)
        );
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
                        doctype_node.set_name(doctype_token.name.unwrap_or_default());
                        doctype_node.set_public_id(doctype_token.public_ident.unwrap_or_default());
                        doctype_node.set_system_id(doctype_token.system_ident.unwrap_or_default());

                        // FIXME: Then, if the document is not an iframe srcdoc document, and the parser cannot change the mode flag is false,
                        // and the DOCTYPE token matches one of the conditions in the following list, then set the Document to quirks mode:
                        let new_node = DomPtr::new(doctype_node).upcast();
                        Node::append_child(DomPtr::clone(&self.document).upcast(), new_node);
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
                            && tagdata.name != static_interned!("head")
                            && tagdata.name != static_interned!("body")
                            && tagdata.name != static_interned!("html")
                            && tagdata.name != static_interned!("br") =>
                    {
                        // Parse error. Ignore the token.
                    },
                    Token::Tag(ref tagdata)
                        if tagdata.opening && tagdata.name == static_interned!("html") =>
                    {
                        // Create an element for the token in the HTML namespace, with the Document as the intended parent.
                        let element = self.create_element_for_token(
                            tagdata,
                            Namespace::HTML,
                            &DomPtr::clone(&self.document).upcast(),
                        );

                        // Append it to the Document object.
                        Node::append_child(
                            DomPtr::clone(&self.document).upcast(),
                            DomPtr::clone(&element).upcast(),
                        );

                        // Put this element in the stack of open elements.
                        self.open_elements.push(element);

                        // Switch the insertion mode to "before head".
                        self.insertion_mode = InsertionMode::BeforeHead;
                    },
                    other => {
                        // Create an html element whose node document is the Document object.
                        let mut html_element = HtmlHtmlElement::default();
                        html_element.set_owning_document(DomPtr::clone(&self.document).downgrade());
                        let new_element: DomPtr<Element> = DomPtr::new(html_element).upcast();

                        // Append it to the Document object.
                        Node::append_child(
                            DomPtr::clone(&self.document).upcast(),
                            DomPtr::clone(&new_element).upcast(),
                        );

                        // Put this element in the stack of open elements.
                        self.open_elements.push(new_element);

                        // Switch the insertion mode to "before head",
                        self.insertion_mode = InsertionMode::BeforeHead;

                        // then reprocess the token.
                        self.consume(other);
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
                    Token::Tag(ref tagdata)
                        if tagdata.opening && tagdata.name == static_interned!("html") =>
                    {
                        // Process the token using the rules for the "in body" insertion mode.
                        self.consume_in_mode(InsertionMode::InBody, token);
                    },
                    Token::Tag(ref tagdata)
                        if tagdata.opening && tagdata.name == static_interned!("head") =>
                    {
                        // Insert an HTML element for the token.
                        let head = self
                            .insert_html_element_for_token(tagdata)
                            .try_into_type()
                            .expect("expected <head> element to be created");

                        // Set the head element pointer to the newly created head element.
                        self.head = Some(head);

                        // Switch the insertion mode to "in head".
                        self.insertion_mode = InsertionMode::InHead;
                    },
                    Token::Tag(ref tagdata)
                        if !tagdata.opening
                            && tagdata.name != static_interned!("head")
                            && tagdata.name != static_interned!("body")
                            && tagdata.name != static_interned!("html")
                            && tagdata.name != static_interned!("br") =>
                    {
                        // Parse error. Ignore the token.
                    },
                    other => {
                        // Insert an HTML element for a "head" start tag token with no attributes.
                        let bogus_head_token = TagData {
                            name: static_interned!("head"),
                            opening: true,
                            self_closing: false,
                            attributes: vec![],
                        };
                        let head_element = self
                            .insert_html_element_for_token(&bogus_head_token)
                            .try_into_type()
                            .expect("expected <head> element to be created");

                        // Set the head element pointer to the newly created head element.
                        self.head = Some(head_element);

                        // Switch the insertion mode to "in head".
                        self.insertion_mode = InsertionMode::InHead;

                        // Reprocess the current token.
                        self.consume(other);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inhead
            InsertionMode::InHead => {
                match token {
                    Token::Character(c @ (TAB | LINE_FEED | FORM_FEED | WHITESPACE)) => {
                        // Insert the character.
                        self.insert_character(c);
                    },
                    Token::Comment(data) => {
                        // Insert a comment.
                        self.insert_comment(data);
                    },
                    Token::DOCTYPE(_) => {
                        // parse error, ignore token
                    },
                    Token::Tag(ref tagdata)
                        if tagdata.opening && tagdata.name == static_interned!("html") =>
                    {
                        // Process the token using the rules for the "in body" insertion mode.
                        self.consume_in_mode(InsertionMode::InBody, token);
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening
                            && (tagdata.name == static_interned!("base")
                                || tagdata.name == static_interned!("basefont")
                                || tagdata.name == static_interned!("bgsound")
                                || tagdata.name == static_interned!("link")) =>
                    {
                        // Insert an HTML element for the token.
                        self.insert_html_element_for_token(&tagdata);

                        // Immediately pop the current node off the stack of open elements.
                        self.pop_from_open_elements();

                        // Acknowledge the token's self-closing flag, if it is set.
                        self.acknowledge_self_closing_flag_if_set(&tagdata);
                    },
                    Token::Tag(ref tagdata)
                        if tagdata.opening && tagdata.name == static_interned!("meta") =>
                    {
                        // Insert an HTML element for the token.
                        self.insert_html_element_for_token(tagdata);

                        // Immediately pop the current node off the stack of open elements.
                        self.pop_from_open_elements();

                        // Acknowledge the token's self-closing flag, if it is set.
                        self.acknowledge_self_closing_flag_if_set(tagdata);

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
                    Token::Tag(tagdata)
                        if tagdata.opening && tagdata.name == static_interned!("title") =>
                    {
                        // Follow the generic RCDATA element parsing algorithm.
                        self.generic_parsing_algorithm(tagdata, GenericParsingAlgorithm::RcData);
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening
                            && tagdata.name == static_interned!("noscript")
                            && self.execute_script =>
                    {
                        // Follow the generic raw text element parsing algorithm.
                        self.generic_parsing_algorithm(tagdata, GenericParsingAlgorithm::RawText);
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening
                            && (tagdata.name == static_interned!("noframes")
                                || tagdata.name == static_interned!("style")) =>
                    {
                        // Follow the generic raw text element parsing algorithm.
                        self.generic_parsing_algorithm(tagdata, GenericParsingAlgorithm::RawText);
                    },
                    Token::Tag(ref tagdata)
                        if tagdata.opening
                            && tagdata.name == static_interned!("noscript")
                            && !self.execute_script =>
                    {
                        // Insert an HTML element for the token.
                        self.insert_html_element_for_token(tagdata);

                        // Switch the insertion mode to "in head noscript".
                        self.insertion_mode = InsertionMode::InHeadNoscript;
                    },
                    Token::Tag(ref tagdata)
                        if tagdata.opening && tagdata.name == static_interned!("script") =>
                    {
                        // 1. Let the adjusted insertion location be the appropriate place for inserting a node.
                        let adjusted_insert_location = self.appropriate_place_for_inserting_node();

                        // 2. Create an element for the token in the HTML namespace, with the intended parent being the element
                        // in which the adjusted insertion location finds itself.
                        let element = self.create_element_for_token(
                            tagdata,
                            Namespace::HTML,
                            &adjusted_insert_location,
                        );

                        // 3. Set the element's parser document to the Document, and set the element's force async to false.
                        log::warn!("FIXME: Set script element attributes");

                        // 4. If the parser was created as part of the HTML fragment parsing algorithm, then set the script element's already started to true. (fragment case)

                        // 5. If the parser was invoked via the document.write() or document.writeln() methods, then optionally set the script element's already started to true.

                        // 6. Insert the newly created element at the adjusted insertion location.
                        Node::append_child(adjusted_insert_location, element.clone().upcast());

                        // 7. Push the element onto the stack of open elements so that it is the new current node.
                        self.open_elements.push(element);

                        // 8. Switch the tokenizer to the script data state.
                        self.tokenizer.switch_to(TokenizerState::ScriptData);

                        // 9. Let the original insertion mode be the current insertion mode.
                        self.original_insertion_mode = Some(self.insertion_mode);

                        // 10. Switch the insertion mode to "text".
                        self.insertion_mode = InsertionMode::Text;
                    },
                    Token::Tag(ref tagdata)
                        if !tagdata.opening && tagdata.name == static_interned!("head") =>
                    {
                        // Pop the current node (which will be the head element) off the stack of open elements.
                        let current_node = self.pop_from_open_elements();
                        assert_eq!(current_node.underlying_type(), DomType::HtmlHeadElement,);

                        // Switch the insertion mode to "after head".
                        self.insertion_mode = InsertionMode::AfterHead;
                    },
                    Token::Tag(ref tagdata)
                        if tagdata.opening && tagdata.name == static_interned!("template") =>
                    {
                        // Insert an HTML element for the token.
                        self.insert_html_element_for_token(tagdata);

                        // Insert a marker at the end of the list of active formatting
                        // elements.
                        self.active_formatting_elements.push_marker();

                        // Set the frameset-ok flag to "not ok".
                        self.frameset_ok = FramesetOkFlag::NotOk;

                        // Switch the insertion mode to "in template".
                        self.insertion_mode = InsertionMode::InTemplate;

                        // Push "in template" onto the stack of template insertion modes so
                        // that it is the new current template insertion mode.
                        self.template_insertion_modes
                            .push(InsertionMode::InTemplate);
                    },
                    Token::Tag(ref tagdata)
                        if !tagdata.opening && tagdata.name == static_interned!("template") =>
                    {
                        // If there is no template element on the stack of open elements, then
                        // this is a parse error; ignore the token.
                        let contains_template_element = self
                            .open_elements
                            .iter()
                            .any(|element| element.is_a::<HtmlTemplateElement>());

                        if !contains_template_element {
                            return;
                        } else {
                            // Otherwise, run these steps:
                            // 1. Generate all implied end tags thoroughly.
                            self.generate_implied_end_tags_thoroughly();

                            // 2. If the current node is not a template element,
                            //    then this is a parse error.

                            // 3. Pop elements from the stack of open
                            //    elements until a template element has
                            //    been popped from the stack.
                            self.pop_from_open_elements_until(|node| {
                                node.is_a::<HtmlTemplateElement>()
                            });

                            // 4. Clear the list of active
                            //    formatting elements up to the last marker.
                            self.active_formatting_elements.clear_up_to_last_marker();

                            // 5. Pop the current template
                            //    insertion mode off the stack of template
                            //    insertion modes.
                            self.template_insertion_modes.pop();

                            // 6. Reset the insertion mode appropriately.
                            self.reset_insertion_mode_appropriately();
                        }
                        todo!();
                    },
                    Token::Tag(ref tagdata)
                        if (tagdata.opening && tagdata.name == static_interned!("head"))
                            || (!tagdata.opening
                                && tagdata.name != static_interned!("body")
                                && tagdata.name != static_interned!("html")
                                && tagdata.name != static_interned!("br")) =>
                    {
                        // Parse error. Ignore the token.
                    },
                    other => {
                        // Pop the current node (which will be the head element) off the stack of open elements.
                        let popped_node = self.pop_from_open_elements();
                        debug_assert_eq!(popped_node.underlying_type(), DomType::HtmlHeadElement);

                        // Switch the insertion mode to "after head".
                        self.insertion_mode = InsertionMode::AfterHead;

                        // Reprocess the token.
                        self.consume(other);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inheadnoscript
            InsertionMode::InHeadNoscript => {
                match token {
                    Token::DOCTYPE(_) => {
                        // Parse error. Ignore the token.
                    },
                    Token::Tag(ref tagdata)
                        if tagdata.opening && tagdata.name == static_interned!("html") =>
                    {
                        // Process the token using the rules for the "in body" insertion
                        // mode.
                        self.consume_in_mode(InsertionMode::InBody, token);
                    },
                    Token::Tag(ref tagdata)
                        if !tagdata.opening && tagdata.name == static_interned!("noscript") =>
                    {
                        // Pop the current node (which will be a noscript element) from the stack of open elements; the new current node will be a head element.
                        let popped_node = self.pop_from_open_elements();
                        debug_assert_eq!(
                            popped_node.underlying_type(),
                            DomType::HtmlNoscriptElement
                        );
                        debug_assert_eq!(
                            self.current_node().underlying_type(),
                            DomType::HtmlHeadElement
                        );

                        // Switch the insertion mode to "in head".
                        self.insertion_mode = InsertionMode::InHead;
                    },
                    Token::Character(TAB | LINE_FEED | FORM_FEED | WHITESPACE)
                    | Token::Comment(_) => {
                        // Process the token using the rules for the "in head" insertion
                        // mode.
                        self.consume_in_mode(InsertionMode::InHead, token);
                    },
                    Token::Tag(ref tagdata)
                        if tagdata.opening
                            && (tagdata.name == static_interned!("basefont")
                                || tagdata.name == static_interned!("bgsound")
                                || tagdata.name == static_interned!("link")
                                || tagdata.name == static_interned!("meta")
                                || tagdata.name == static_interned!("style")) =>
                    {
                        // Process the token using the rules for the "in head" insertion
                        // mode.
                        self.consume_in_mode(InsertionMode::InHead, token);
                    },
                    Token::Tag(ref tagdata)
                        if tagdata.opening
                            && (tagdata.name == static_interned!("head")
                                || tagdata.name == static_interned!("noscript")) => {}, // Parse error. Ignore the token.
                    Token::Tag(ref tagdata)
                        if !tagdata.opening && (tagdata.name != static_interned!("br")) => {}, // Parse error. Ignore the token.
                    other => {
                        // Parse error.

                        // Pop the current node (which will be a noscript element) from the stack of open elements; the new current node will be a head element.
                        let popped_node = self.pop_from_open_elements();
                        debug_assert_eq!(
                            popped_node.underlying_type(),
                            DomType::HtmlNoscriptElement
                        );
                        debug_assert_eq!(
                            self.current_node().underlying_type(),
                            DomType::HtmlHeadElement
                        );

                        // Switch the insertion mode to "in head".
                        self.insertion_mode = InsertionMode::InHead;

                        // Reprocess the token.
                        self.consume(other);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#the-after-head-insertion-mode
            InsertionMode::AfterHead => {
                match token {
                    Token::Character(c @ (TAB | LINE_FEED | FORM_FEED | WHITESPACE)) => {
                        // Insert the character.
                        self.insert_character(c);
                    },
                    Token::Comment(data) => {
                        // Insert a comment.
                        self.insert_comment(data);
                    },
                    Token::DOCTYPE(_) => {}, // Parse error. Ignore the token.
                    Token::Tag(ref tagdata)
                        if tagdata.opening && tagdata.name == static_interned!("html") =>
                    {
                        // Process the token using the rules for the "in body" insertion mode.
                        self.consume_in_mode(InsertionMode::InBody, token);
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening && tagdata.name == static_interned!("body") =>
                    {
                        // Insert an HTML element for the token.
                        self.insert_html_element_for_token(&tagdata);

                        // Set the frameset-ok flag to "not ok".

                        // Switch the insertion mode to "in body".
                        self.insertion_mode = InsertionMode::InBody;
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening && tagdata.name == static_interned!("frameset") =>
                    {
                        // Insert an HTML element for the token.
                        self.insert_html_element_for_token(&tagdata);

                        // Switch the insertion mode to "in frameset".
                        self.insertion_mode = InsertionMode::InFrameset;
                    },
                    Token::Tag(ref tagdata)
                        if tagdata.opening
                            && (tagdata.name == static_interned!("base")
                                || tagdata.name == static_interned!("basefont")
                                || tagdata.name == static_interned!("bgsound")
                                || tagdata.name == static_interned!("link")
                                || tagdata.name == static_interned!("meta")
                                || tagdata.name == static_interned!("noframes")
                                || tagdata.name == static_interned!("script")
                                || tagdata.name == static_interned!("style")
                                || tagdata.name == static_interned!("template")
                                || tagdata.name == static_interned!("title")) =>
                    {
                        // Parse error.

                        // Push the node pointed to by the head element pointer onto the stack of open elements.
                        let head = DomPtr::clone(
                            self.head
                                .as_ref()
                                .expect("Spec: self.head cannot be none at this point"),
                        );
                        self.open_elements.push(DomPtr::clone(&head).upcast());

                        // Process the token using the rules for the "in head" insertion mode.
                        self.consume_in_mode(InsertionMode::InHead, token);

                        // Remove the node pointed to by the head element pointer from the stack of open elements. (It might not be the current node at this point.)
                        self.remove_from_open_elements(&head);
                    },
                    Token::Tag(ref tagdata)
                        if !tagdata.opening && tagdata.name == static_interned!("template") =>
                    {
                        // Process the token using the rules for the "in head" insertion mode.
                        self.consume_in_mode(InsertionMode::InHead, token);
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening && tagdata.name == static_interned!("head") => {}, // Parse error. Ignore the token.
                    Token::Tag(tagdata)
                        if !tagdata.opening
                            && tagdata.name != static_interned!("body")
                            && tagdata.name != static_interned!("html")
                            && tagdata.name != static_interned!("br") => {}, // Parse error. Ignore the token.
                    _ => {
                        // Insert an HTML element for a "body" start tag token with no attributes.
                        let body_token = TagData {
                            opening: true,
                            name: static_interned!("body"),
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
                        self.reconstruct_active_formatting_elements();

                        // Insert the token's character.
                        self.insert_character(c);
                    },
                    Token::Comment(data) => {
                        // Insert a comment.
                        self.insert_comment(data);
                    },
                    Token::DOCTYPE(_) => {}, // Parse error. Ignore the token.
                    Token::Tag(tagdata)
                        if tagdata.opening && tagdata.name == static_interned!("html") =>
                    {
                        // Parse error.

                        // If there is a template element on the stack of open elements, then ignore
                        // the token.
                        if self.current_node().underlying_type() != DomType::HtmlTemplateElement {
                            // Otherwise, for each attribute on the token, check to see if the attribute is
                            // already present on the top element of the stack of open elements. If it is
                            // not, add the attribute and its corresponding value to that element.
                            todo!();
                        }
                    },
                    Token::Tag(ref tagdata)
                        if (tagdata.opening
                            && (tagdata.name == static_interned!("base")
                                || tagdata.name == static_interned!("basefont")
                                || tagdata.name == static_interned!("bgsound")
                                || tagdata.name == static_interned!("link")
                                || tagdata.name == static_interned!("meta")
                                || tagdata.name == static_interned!("noframes")
                                || tagdata.name == static_interned!("script")
                                || tagdata.name == static_interned!("style")
                                || tagdata.name == static_interned!("template")
                                || tagdata.name == static_interned!("title")))
                            || (!tagdata.opening
                                && tagdata.name == static_interned!("template")) =>
                    {
                        // Process the token using the rules for the "in head" insertion mode.
                        self.consume_in_mode(InsertionMode::InHead, token);
                    },
                    Token::Tag(ref tagdata)
                        if tagdata.opening && tagdata.name == static_interned!("body") =>
                    {
                        // Parse error.

                        // If the second element on the stack of open elements is not a body element,
                        // if the stack of open elements has only one node on it, or if there is a
                        // template element on the stack of open elements, then ignore the token.
                        // (fragment case)
                        let previous_body: DomPtr<Element> = match self.open_elements.get(2) {
                            Some(node) => {
                                if let Some(html_body_element) =
                                    node.try_into_type::<HtmlBodyElement>()
                                {
                                    // FIXME: Check if there's a template element on the stack of open elements
                                    html_body_element.clone().upcast()
                                } else {
                                    return;
                                }
                            },
                            None => return,
                        };
                        {
                            // Otherwise, set the frameset-ok flag to "not ok"; then, for each attribute on
                            // the token, check to see if the attribute is already present on the body
                            // element (the second element) on the stack of open elements, and if it is
                            // not, add the attribute and its corresponding value to that element.
                            self.frameset_ok = FramesetOkFlag::NotOk;

                            let mut previous_body = previous_body.borrow_mut();
                            let attributes = previous_body.attributes_mut();
                            for (key, value) in tagdata.attributes() {
                                attributes.entry(*key).or_insert(*value);
                            }
                        }
                    },
                    Token::Tag(ref tagdata)
                        if tagdata.opening && tagdata.name == static_interned!("frameset") =>
                    {
                        todo!()
                    },
                    Token::EOF => {
                        // If the stack of template insertion modes is not empty, then process the
                        // token using the rules for the "in template" insertion mode.
                        if !self.template_insertion_modes.is_empty() {
                            self.consume_in_mode(InsertionMode::InTemplate, Token::EOF);
                        } else {
                            // Otherwise, follow these steps:
                            // 1. If there is a node in the stack of open elements that is not either
                            //     a dd element, a dt element, an li element, an optgroup element, an
                            //     option element, a p element, an rb element, an rp element, an rt
                            //     element, an rtc element, a tbody element, a td element, a tfoot
                            //     element, a th element, a thead element, a tr element, the body
                            //     element, or the html element, then this is a parse error.
                            //
                            // 2. Stop parsing.
                            self.stop_parsing();
                        }
                    },
                    Token::Tag(ref tagdata)
                        if !tagdata.opening && tagdata.name == static_interned!("body") =>
                    {
                        // If the stack of open elements does not have a body element in scope, this is a parse error; ignore the token.
                        if !self.is_element_in_scope(static_interned!("body")) {
                            return;
                        }

                        // FIXME: Otherwise, if there is a node in the stack of open elements that is not either
                        // a dd element, a dt element, an li element, an optgroup element, an option element,
                        // a p element, an rb element, an rp element, an rt element, an rtc element, a tbody element,
                        // a td element, a tfoot element, a th element, a thead element, a tr element,
                        // the body element, or the html element, then this is a parse error.

                        // Switch the insertion mode to "after body".
                        self.insertion_mode = InsertionMode::AfterBody;
                    },

                    Token::Tag(tagdata)
                        if !tagdata.opening && tagdata.name == static_interned!("html") =>
                    {
                        // If the stack of open elements does not have a body element in scope, this is a parse error; ignore the token.
                        if self.is_element_in_scope(static_interned!("html")) {
                            return;
                        }

                        // Otherwise, if there is a node in the stack of open elements that is not either a dd element,
                        // a dt element, an li element, an optgroup element, an option element, a p element, an rb element,
                        // an rp element, an rt element, an rtc element, a tbody element, a td element, a tfoot element,
                        // a th element, a thead element, a tr element, the body element, or the html element, then this is a parse error.

                        // Switch the insertion mode to "after body".
                        self.insertion_mode = InsertionMode::AfterBody;

                        // Reprocess the token.
                        self.consume(Token::Tag(tagdata));
                    },
                    Token::Tag(ref tagdata)
                        if tagdata.opening
                            && (tagdata.name == static_interned!("address")
                                || tagdata.name == static_interned!("article")
                                || tagdata.name == static_interned!("aside")
                                || tagdata.name == static_interned!("blockquote")
                                || tagdata.name == static_interned!("center")
                                || tagdata.name == static_interned!("details")
                                || tagdata.name == static_interned!("dialog")
                                || tagdata.name == static_interned!("dir")
                                || tagdata.name == static_interned!("div")
                                || tagdata.name == static_interned!("dl")
                                || tagdata.name == static_interned!("fieldset")
                                || tagdata.name == static_interned!("figcaption")
                                || tagdata.name == static_interned!("figure")
                                || tagdata.name == static_interned!("footer")
                                || tagdata.name == static_interned!("header")
                                || tagdata.name == static_interned!("hgroup")
                                || tagdata.name == static_interned!("main")
                                || tagdata.name == static_interned!("menu")
                                || tagdata.name == static_interned!("nav")
                                || tagdata.name == static_interned!("ol")
                                || tagdata.name == static_interned!("p")
                                || tagdata.name == static_interned!("section")
                                || tagdata.name == static_interned!("summary")
                                || tagdata.name == static_interned!("ul")) =>
                    {
                        // If the stack of open elements has a p element in button scope, then close a p element.
                        if self.is_element_in_button_scope(static_interned!("p")) {
                            self.close_p_element();
                        }

                        // Insert an HTML element for the token.
                        self.insert_html_element_for_token(tagdata);
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening
                            && (tagdata.name == static_interned!("h1")
                                || tagdata.name == static_interned!("h2")
                                || tagdata.name == static_interned!("h3")
                                || tagdata.name == static_interned!("h4")
                                || tagdata.name == static_interned!("h5")
                                || tagdata.name == static_interned!("h6")) =>
                    {
                        // If the stack of open elements has a p element in button scope, then close a p element.
                        if self.is_element_in_button_scope(static_interned!("p")) {
                            self.close_p_element();
                        }

                        // If the current node is an HTML element whose tag name is one of
                        // "h1", "h2", "h3", "h4", "h5", or "h6", then this is a parse error;
                        // pop the current node off the stack of open elements.
                        let tag_name = self.current_node().borrow().local_name();
                        if matches!(
                            tag_name,
                            static_interned!("h1")
                                | static_interned!("h2")
                                | static_interned!("h3")
                                | static_interned!("h4")
                                | static_interned!("h5")
                                | static_interned!("h6")
                        ) {
                            self.pop_from_open_elements();
                        }

                        // Insert an HTML element for the token.
                        self.insert_html_element_for_token(&tagdata);
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening
                            && (tagdata.name == static_interned!("pre")
                                || tagdata.name == static_interned!("listing")) =>
                    {
                        // If the stack of open elements has a p element in button scope, then close a p element.
                        if self.is_element_in_scope(static_interned!("p")) {
                            self.close_p_element();
                        }

                        // Insert an HTML element for the token.
                        self.insert_html_element_for_token(&tagdata);

                        // FIXME: If the next token is a U+000A LINE FEED (LF) character token, then ignore that token
                        // and move on to the next one. (Newlines at the start of pre blocks are ignored as an authoring convenience.)

                        // Set the frameset-ok flag to "not ok".
                        self.frameset_ok = FramesetOkFlag::NotOk;
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening && tagdata.name == static_interned!("form") =>
                    {
                        // If the form element pointer is not null, and there is no template element on the stack of open elements,
                        // then this is a parse error; ignore the token.
                        let is_template_on_open_elements = self
                            .open_elements
                            .iter()
                            .any(|element| element.is_a::<HtmlTemplateElement>());
                        if self.form.is_some() && !is_template_on_open_elements {
                            #[allow(clippy::needless_return)]
                            return;
                        }
                        // Otherwise:
                        else {
                            // If the stack of open elements has a p element in button scope, then close a p element.
                            if self.is_element_in_button_scope(static_interned!("p")) {
                                self.close_p_element();
                            }

                            // Insert an HTML element for the token, and, if there is no template element on the
                            // stack of open elements, set the form element pointer to point to the element created.
                            let new_element = self
                                .insert_html_element_for_token(&tagdata)
                                .try_into_type::<HtmlFormElement>()
                                .unwrap();
                            if !is_template_on_open_elements {
                                self.form = Some(new_element);
                            }
                        }
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening && tagdata.name == static_interned!("li") =>
                    {
                        // Run these steps:
                        // 1. Set the frameset-ok flag to "not ok".
                        self.frameset_ok = FramesetOkFlag::NotOk;

                        // 2. Initialize node to be the current node (the bottommost node of the stack).
                        let mut node_index = self.open_elements.len() - 1;
                        let mut node = self.open_elements[node_index].clone();

                        loop {
                            // 3. Loop: If node is an li element, then run these substeps:
                            if node.is_a::<HtmlLiElement>() {
                                log::warn!("FIXME: Handle nested <li> elements");

                                // 4. Jump to the step labeled done below.
                                break;
                            }

                            // 4. If node is in the special category, but is not an address, div, or p element,
                            //    then jump to the step labeled done below.
                            let local_name = node.clone().borrow().local_name();

                            // FIXME: Check if node is an address element
                            if is_element_in_special_category(local_name)
                                && !(node.is_a::<HtmlDivElement>()
                                    || node.is_a::<HtmlParagraphElement>())
                            {
                                break;
                            } else {
                                // 5. Otherwise, set node to the previous entry in the stack of open elements
                                //    and return to the step labeled loop.
                                node_index -= 1;
                                node = self.open_elements[node_index].clone();
                            }
                        }

                        // 6. Done: If the stack of open elements has a p element in button scope, then close a p element.
                        if self.is_element_in_button_scope(static_interned!("p")) {
                            self.close_p_element();
                        }

                        // 7. Finally, insert an HTML element for the token.
                        self.insert_html_element_for_token(&tagdata);
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening
                            && (tagdata.name == static_interned!("dd")
                                || tagdata.name == static_interned!("dt")) =>
                    {
                        // Run these steps:
                        // 1. Set the frameset-ok flag to "not ok".
                        self.frameset_ok = FramesetOkFlag::NotOk;

                        // 2. Initialize node to be the current node (the bottommost node of the stack).
                        let node = self.current_node();

                        // 3. Loop:
                        // If node is a dd element, then run these substeps:
                        if node.is_a::<HtmlDdElement>() {
                            // 1. Generate implied end tags, except for dd elements.
                            self.generate_implied_end_tags_excluding(Some(static_interned!("dd")));

                            // 2. If the current node is not a dd element, then this is a parse error.

                            // 3. Pop elements from the stack of open elements until a dd element has been popped from the stack.
                            self.pop_from_open_elements_until(|node| node.is_a::<HtmlDdElement>());

                            // 4. Jump to the step labeled done below.
                            // break;
                        }
                        // FIXME: 4. If node is a dt element, then run these substeps:
                        // FIXME: 5. If node is in the special category, but is not an address, div, or p element, then jump to the step labeled done below.
                        // FIXME: 6. Otherwise, set node to the previous entry in the stack of open elements and return to the step labeled loop.

                        // 7. Done: If the stack of open elements has a p element in button scope, then close a p element.
                        if self.is_element_in_scope(static_interned!("p")) {
                            self.close_p_element();
                        }

                        // 8. Finally, insert an HTML element for the token.
                        self.insert_html_element_for_token(&tagdata);
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening && tagdata.name == static_interned!("plaintext") =>
                    {
                        // If the stack of open elements has a p element in button scope, then close a p element.
                        if self.is_element_in_button_scope(static_interned!("p")) {
                            self.close_p_element();
                        }

                        // Insert an HTML element for the token.
                        self.insert_html_element_for_token(&tagdata);

                        // Switch the tokenizer to the PLAINTEXT state.
                        self.tokenizer.switch_to(TokenizerState::PLAINTEXT);
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening && tagdata.name == static_interned!("button") =>
                    {
                        // 1. If the stack of open elements has a button element in scope, then run these substeps:
                        if self.is_element_in_scope(static_interned!("button")) {
                            // 1. Parse error.
                            // 2. Generate implied end tags.
                            self.generate_implied_end_tags();

                            // 3. Pop elements from the stack of open elements until a button element has been popped from the stack.
                            self.pop_from_open_elements_until(|elem| {
                                elem.borrow().local_name() == static_interned!("button")
                            });
                        }

                        // 2. Reconstruct the active formatting elements, if any.
                        self.reconstruct_active_formatting_elements();

                        // 3. Insert an HTML element for the token.
                        self.insert_html_element_for_token(&tagdata);

                        // 4. Set the frameset-ok flag to "not ok".
                        self.frameset_ok = FramesetOkFlag::NotOk;
                    },
                    Token::Tag(ref tagdata)
                        if tagdata.opening
                            && (tagdata.name == static_interned!("address")
                                || tagdata.name == static_interned!("article")
                                || tagdata.name == static_interned!("aside")
                                || tagdata.name == static_interned!("blockquote")
                                || tagdata.name == static_interned!("center")
                                || tagdata.name == static_interned!("details")
                                || tagdata.name == static_interned!("dialog")
                                || tagdata.name == static_interned!("dir")
                                || tagdata.name == static_interned!("div")
                                || tagdata.name == static_interned!("dl")
                                || tagdata.name == static_interned!("fieldset")
                                || tagdata.name == static_interned!("figcaption")
                                || tagdata.name == static_interned!("figure")
                                || tagdata.name == static_interned!("footer")
                                || tagdata.name == static_interned!("header")
                                || tagdata.name == static_interned!("hgroup")
                                || tagdata.name == static_interned!("main")
                                || tagdata.name == static_interned!("menu")
                                || tagdata.name == static_interned!("nav")
                                || tagdata.name == static_interned!("ol")
                                || tagdata.name == static_interned!("p")
                                || tagdata.name == static_interned!("section")
                                || tagdata.name == static_interned!("summary")
                                || tagdata.name == static_interned!("ul")) =>
                    {
                        // If the stack of open elements does not have an element in scope that is an HTML element with
                        // the same tag name as that of the token, then this is a parse error; ignore the token.
                        if self.is_element_in_scope(tagdata.name) {
                            return;
                        }

                        // Otherwise, run these steps:
                        // 1. Generate implied end tags.
                        self.generate_implied_end_tags();

                        // 2. If the current node is not an HTML element with the same tag name as that of the token,
                        //    then this is a parse error.

                        // 3. Pop elements from the stack of open elements until an HTML element
                        //    with the same tag name as the token has been popped from the stack.
                        self.pop_from_open_elements_until(|elem| {
                            elem.borrow().local_name() == tagdata.name
                        });
                    },
                    Token::Tag(ref tagdata)
                        if !tagdata.opening && tagdata.name == static_interned!("form") =>
                    {
                        // If there is no template element on the stack of open elements, then run these substeps:
                        let is_template_on_open_elements = self
                            .open_elements
                            .iter()
                            .any(|element| element.is_a::<HtmlTemplateElement>());
                        if !is_template_on_open_elements {
                            // 1. Let node be the element that the form element pointer is set to, or null if it is not set to an element.
                            // 2. Set the form element pointer to null.
                            let node = self.form.take();

                            match node {
                                Some(node)
                                    if self.is_element_in_scope(node.borrow().local_name()) =>
                                {
                                    // 4. Generate implied end tags.
                                    self.generate_implied_end_tags();

                                    // 5. If the current node is not node, then this is a parse error.
                                    self.remove_from_open_elements(&node);
                                },
                                None | Some(_) => {
                                    // 3. If node is null or if the stack of open elements does not have node in scope,
                                    //    then this is a parse error; return and ignore the token.
                                    #[allow(clippy::needless_return)]
                                    return;
                                },
                            }
                        }
                        // If there is a template element on the stack of open elements, then run these substeps instead:
                        else {
                            // 1. If the stack of open elements does not have a form element in scope, then this is a parse error; return and ignore the token.
                            if !self.is_element_in_scope(static_interned!("form")) {
                                return;
                            }

                            // 2. Generate implied end tags.
                            self.generate_implied_end_tags();

                            // 3. If the current node is not a form element, then this is a parse error.

                            // 4. Pop elements from the stack of open elements until a form element has been popped from the stack.
                            self.pop_from_open_elements_until(|node| {
                                node.is_a::<HtmlFormElement>()
                            });
                        }
                    },
                    Token::Tag(ref tagdata)
                        if !tagdata.opening && tagdata.name == static_interned!("p") =>
                    {
                        // If the stack of open elements does not have a p element in button scope, then this is a parse error;
                        // insert an HTML element for a "p" start tag token with no attributes.
                        if !self.is_element_in_button_scope(static_interned!("p")) {
                            self.insert_html_element_for_token(&TagData {
                                opening: true,
                                name: static_interned!("p"),
                                self_closing: false,
                                attributes: vec![],
                            });
                        }

                        // Close a p element.
                        self.close_p_element();
                    },
                    Token::Tag(ref tagdata)
                        if !tagdata.opening && tagdata.name == static_interned!("li") =>
                    {
                        // If the stack of open elements does not have an li element in list item scope,
                        // then this is a parse error; ignore the token.
                        if !self.is_element_in_list_item_scope(static_interned!("li")) {
                            return;
                        }

                        // Otherwise, run these steps:
                        // 1. Generate implied end tags, except for li elements.
                        self.generate_implied_end_tags_excluding(Some(static_interned!("li")));

                        // 2. If the current node is not an li element, then this is a parse error.

                        // 3. Pop elements from the stack of open elements until an li element has been popped from the stack.
                        self.pop_from_open_elements_until(|node| node.is_a::<HtmlLiElement>());
                    },
                    Token::Tag(ref tagdata)
                        if !tagdata.opening
                            && (tagdata.name == static_interned!("dd")
                                || tagdata.name == static_interned!("dt")) =>
                    {
                        // If the stack of open elements does not have an element in scope that is an HTML element with the
                        // same tag name as that of the token, then this is a parse error; ignore the token.
                        if !self.is_element_in_scope(tagdata.name) {
                            return;
                        }

                        // Otherwise, run these steps:
                        // 1. Generate implied end tags, except for HTML elements with the same tag name as the token.
                        self.generate_implied_end_tags_excluding(Some(tagdata.name));

                        // 2. If the current node is not an HTML element with the same tag name as that of the token, then this is a parse error.

                        // 3. Pop elements from the stack of open elements until an HTML element with the same tag name as the token has been popped from the stack.
                        self.pop_from_open_elements_until(|node| {
                            node.try_into_type::<Element>().is_some_and(|element| {
                                element.borrow().local_name() == tagdata.name
                            })
                        });
                    },
                    Token::Tag(ref tagdata)
                        if !tagdata.opening
                            && (tagdata.name == static_interned!("h1")
                                || tagdata.name == static_interned!("h2")
                                || tagdata.name == static_interned!("h3")
                                || tagdata.name == static_interned!("h4")
                                || tagdata.name == static_interned!("h5")
                                || tagdata.name == static_interned!("h6")) =>
                    {
                        // FIXME: If the stack of open elements does not have an element in scope that is an HTML element
                        //        and whose tag name is one of "h1", "h2", "h3", "h4", "h5", or "h6",
                        //        then this is a parse error; ignore the token.
                        let is_heading_in_scope = self.elements_in_scope(DEFAULT_SCOPE).any(|e| {
                            let local_name = e.borrow().local_name();
                            matches!(
                                local_name,
                                static_interned!("h1")
                                    | static_interned!("h2")
                                    | static_interned!("h3")
                                    | static_interned!("h4")
                                    | static_interned!("h5")
                                    | static_interned!("h6")
                            )
                        });
                        if !is_heading_in_scope {
                            return;
                        }

                        // Otherwise, run these steps:

                        // 1. Generate implied end tags.
                        self.generate_implied_end_tags();

                        // 2. If the current node is not an HTML element with the same tag name as that of the token,
                        //    then this is a parse error.

                        // 3. Pop elements from the stack of open elements until an HTML element whose tag name is one of
                        //    "h1", "h2", "h3", "h4", "h5", or "h6" has been popped from the stack.
                        self.pop_from_open_elements_until(|node| {
                            if let Some(element) = node.try_into_type::<Element>() {
                                let tag_name = element.borrow().local_name();
                                matches!(
                                    tag_name,
                                    static_interned!("h1")
                                        | static_interned!("h2")
                                        | static_interned!("h3")
                                        | static_interned!("h4")
                                        | static_interned!("h5")
                                        | static_interned!("h6")
                                )
                            } else {
                                false
                            }
                        });
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening && tagdata.name == static_interned!("a") =>
                    {
                        // If the list of active formatting elements contains an a element between the end of the list and the last marker on the list
                        // (or the start of the list if there is no marker on the list)
                        let a_element = self
                            .active_formatting_elements
                            .elements_since_last_marker()
                            .map(|formatting_element| formatting_element.element)
                            .find(|element| {
                                element.underlying_type() == DomType::HtmlAnchorElement
                            });
                        if let Some(element) = a_element {
                            // then this is a parse error;
                            // run the adoption agency algorithm for the token,
                            self.run_adoption_agency_algorithm(&tagdata);

                            // then remove that element from the list of active formatting elements
                            // and the stack of open elements if the adoption agency algorithm didn't already remove it
                            // (it might not have if the element is not in table scope).
                            self.active_formatting_elements.remove(&element);
                            self.remove_from_open_elements(&element);
                            log::warn!("FIXME: adoption agency algorithm for invalid <a> tag");
                        }

                        // Reconstruct the active formatting elements, if any.
                        self.reconstruct_active_formatting_elements();

                        // Insert an HTML element for the token.
                        let element = self.insert_html_element_for_token(&tagdata);

                        // Push onto the list of active formatting elements that element.
                        // NOTE: the cast is safe because "insert_html_element_for_token" alwas produces an HTMLElement
                        self.active_formatting_elements.push(element, tagdata);
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening
                            && (tagdata.name == static_interned!("b")
                                || tagdata.name == static_interned!("big")
                                || tagdata.name == static_interned!("code")
                                || tagdata.name == static_interned!("em")
                                || tagdata.name == static_interned!("font")
                                || tagdata.name == static_interned!("i")
                                || tagdata.name == static_interned!("s")
                                || tagdata.name == static_interned!("small")
                                || tagdata.name == static_interned!("strike")
                                || tagdata.name == static_interned!("strong")
                                || tagdata.name == static_interned!("tt")
                                || tagdata.name == static_interned!("u")) =>
                    {
                        // Reconstruct the active formatting elements, if any.
                        self.reconstruct_active_formatting_elements();

                        // Insert an HTML element for the token
                        let element = self.insert_html_element_for_token(&tagdata);

                        // Push onto the list of active formatting elements that element.
                        self.active_formatting_elements.push(element, tagdata);
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening && tagdata.name == static_interned!("nobr") =>
                    {
                        // Reconstruct the active formatting elements, if any.
                        self.reconstruct_active_formatting_elements();

                        // If the stack of open elements has a nobr element in scope, then this is a parse error;
                        // run the adoption agency algorithm for the token, then once again reconstruct the
                        // active formatting elements, if any.
                        if !self.is_element_in_scope(static_interned!("nobr")) {
                            self.run_adoption_agency_algorithm(&tagdata);
                            self.reconstruct_active_formatting_elements();
                        }

                        // Insert an HTML element for the token
                        let element = self.insert_html_element_for_token(&tagdata);

                        // Push onto the list of active formatting elements that element.
                        self.active_formatting_elements.push(element, tagdata);
                    },
                    Token::Tag(tagdata)
                        if !tagdata.opening
                            && (tagdata.name == static_interned!("a")
                                || tagdata.name == static_interned!("b")
                                || tagdata.name == static_interned!("big")
                                || tagdata.name == static_interned!("code")
                                || tagdata.name == static_interned!("em")
                                || tagdata.name == static_interned!("font")
                                || tagdata.name == static_interned!("i")
                                || tagdata.name == static_interned!("nobr")
                                || tagdata.name == static_interned!("s")
                                || tagdata.name == static_interned!("small")
                                || tagdata.name == static_interned!("strike")
                                || tagdata.name == static_interned!("strong")
                                || tagdata.name == static_interned!("tt")
                                || tagdata.name == static_interned!("u")) =>
                    {
                        // Run the adoption agency algorithm for the token.
                        self.run_adoption_agency_algorithm(&tagdata);
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening
                            && (tagdata.name == static_interned!("applet")
                                || tagdata.name == static_interned!("marquee")
                                || tagdata.name == static_interned!("object")) =>
                    {
                        // Reconstruct the active formatting elements, if any.
                        self.reconstruct_active_formatting_elements();

                        // Insert an HTML element for the token
                        self.insert_html_element_for_token(&tagdata);

                        // Insert a marker at the end of the list of active formatting elements.
                        self.active_formatting_elements.push_marker();

                        // Set the frameset-ok flag to "not ok".
                        self.frameset_ok = FramesetOkFlag::NotOk;
                    },
                    Token::Tag(tagdata)
                        if !tagdata.opening
                            && (tagdata.name == static_interned!("applet")
                                || tagdata.name == static_interned!("marquee")
                                || tagdata.name == static_interned!("object")) =>
                    {
                        todo!();
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening && tagdata.name == static_interned!("table") =>
                    {
                        // If the Document is not set to quirks mode, and the stack of open elements has a p element in button scope, then close a p element.
                        // FIXME: respect quirks mode
                        if self.is_element_in_scope(static_interned!("p")) {
                            self.close_p_element();
                        }

                        // Insert an HTML element for the token.
                        self.insert_html_element_for_token(&tagdata);

                        // Set the frameset-ok flag to "not ok".
                        self.frameset_ok = FramesetOkFlag::NotOk;

                        // Switch the insertion mode to "in table".
                        self.insertion_mode = InsertionMode::InTable;
                    },
                    Token::Tag(mut tagdata)
                        if !tagdata.opening && tagdata.name == static_interned!("br") =>
                    {
                        // Parse error. Drop the attributes from the token, and act as described in the next entry; i.e.
                        // act as if this was a "br" start tag token with no attributes, rather than the end tag token that it actually is.
                        tagdata.attributes.clear();

                        // Reconstruct the active formatting elements, if any.
                        self.reconstruct_active_formatting_elements();

                        // Insert an HTML element for the token.
                        self.insert_html_element_for_token(&tagdata);

                        // Immediately pop the current node off the stack of open elements.
                        self.pop_from_open_elements();

                        // Acknowledge the token's self-closing flag, if it is set.
                        self.acknowledge_self_closing_flag_if_set(&tagdata);

                        // Set the frameset-ok flag to "not ok".
                        self.frameset_ok = FramesetOkFlag::NotOk;
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening
                            && (tagdata.name == static_interned!("area")
                                || tagdata.name == static_interned!("br")
                                || tagdata.name == static_interned!("embed")
                                || tagdata.name == static_interned!("img")
                                || tagdata.name == static_interned!("keygen")
                                || tagdata.name == static_interned!("wbr")) =>
                    {
                        // Reconstruct the active formatting elements, if any.
                        self.reconstruct_active_formatting_elements();

                        // Insert an HTML element for the token.
                        self.insert_html_element_for_token(&tagdata);

                        // Immediately pop the current node off the stack of open elements.
                        self.pop_from_open_elements();

                        // Acknowledge the token's self-closing flag, if it is set.
                        self.acknowledge_self_closing_flag_if_set(&tagdata);

                        // Set the frameset-ok flag to "not ok".
                        self.frameset_ok = FramesetOkFlag::NotOk;
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening && tagdata.name == static_interned!("input") =>
                    {
                        // Reconstruct the active formatting elements, if any.
                        self.reconstruct_active_formatting_elements();

                        // Insert an HTML element for the token.
                        self.insert_html_element_for_token(&tagdata);

                        // Immediately pop the current node off the stack of open elements.
                        self.pop_from_open_elements();

                        // Acknowledge the token's self-closing flag, if it is set.
                        self.acknowledge_self_closing_flag_if_set(&tagdata);

                        // If the token does not have an attribute with the name "type", or if it does,
                        // but that attribute's value is not an ASCII case-insensitive match for the string "hidden",
                        // then: set the frameset-ok flag to "not ok".
                        if !tagdata.attributes.iter().any(|(key, value)| {
                            *key == static_interned!("type")
                                && value.to_string().eq_ignore_ascii_case("hidden")
                        }) {
                            self.frameset_ok = FramesetOkFlag::NotOk;
                        }
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening
                            && (tagdata.name == static_interned!("param")
                                || tagdata.name == static_interned!("source")
                                || tagdata.name == static_interned!("track")) =>
                    {
                        // Insert an HTML element for the token.
                        self.insert_html_element_for_token(&tagdata);

                        // Immediately pop the current node off the stack of open elements.
                        self.pop_from_open_elements();

                        // Acknowledge the token's self-closing flag, if it is set.
                        self.acknowledge_self_closing_flag_if_set(&tagdata);
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening && tagdata.name == static_interned!("hr") =>
                    {
                        // If the stack of open elements has a p element in button scope, then close a p element.
                        if self.is_element_in_button_scope(static_interned!("p")) {
                            self.close_p_element();
                        }

                        // Insert an HTML element for the token.
                        self.insert_html_element_for_token(&tagdata);

                        // Immediately pop the current node off the stack of open elements.
                        self.pop_from_open_elements();

                        // Acknowledge the token's self-closing flag, if it is set.
                        self.acknowledge_self_closing_flag_if_set(&tagdata);

                        // Set the frameset-ok flag to "not ok".
                        self.frameset_ok = FramesetOkFlag::NotOk;
                    },
                    Token::Tag(mut tagdata)
                        if tagdata.opening && tagdata.name == static_interned!("image") =>
                    {
                        // Parse error. Change the token's tag name to "img" and reprocess it. (Don't ask.)
                        tagdata.name = static_interned!("img");
                        self.consume(Token::Tag(tagdata));
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening && tagdata.name == static_interned!("textarea") =>
                    {
                        // Run these steps:

                        // 1. Insert an HTML element for the token.
                        self.insert_html_element_for_token(&tagdata);

                        // 2. If the next token is a U+000A LINE FEED (LF) character token, then ignore that token and move on to the next one.
                        //    (Newlines at the start of textarea elements are ignored as an authoring convenience.)
                        log::warn!("FIXME: ignore newlines at start of <textarea>");

                        // 3. Switch the tokenizer to the RCDATA state.
                        self.tokenizer.switch_to(TokenizerState::RCDATA);

                        // 4. Let the original insertion mode be the current insertion mode.
                        self.original_insertion_mode = Some(self.insertion_mode);

                        // 5. Set the frameset-ok flag to "not ok".
                        self.frameset_ok = FramesetOkFlag::NotOk;

                        // 6. Switch the insertion mode to "text".
                        self.insertion_mode = InsertionMode::Text;
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening && tagdata.name == static_interned!("xmp") =>
                    {
                        // If the stack of open elements has a p element in button scope, then close a p element.
                        if self.is_element_in_button_scope(static_interned!("p")) {
                            self.close_p_element();
                        }

                        // Reconstruct the active formatting elements, if any.
                        self.reconstruct_active_formatting_elements();

                        // Set the frameset-ok flag to "not ok".
                        self.frameset_ok = FramesetOkFlag::NotOk;

                        // Follow the generic raw text element parsing algorithm.
                        self.generic_parsing_algorithm(tagdata, GenericParsingAlgorithm::RawText);
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening && tagdata.name == static_interned!("iframe") =>
                    {
                        // Set the frameset-ok flag to "not ok".
                        self.frameset_ok = FramesetOkFlag::NotOk;

                        // Follow the generic raw text element parsing algorithm.
                        self.generic_parsing_algorithm(tagdata, GenericParsingAlgorithm::RawText);
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening && tagdata.name == static_interned!("noembed") =>
                    {
                        // Follow the generic raw text element parsing algorithm.
                        self.generic_parsing_algorithm(tagdata, GenericParsingAlgorithm::RawText);
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening
                            && tagdata.name == static_interned!("noscript")
                            && self.execute_script =>
                    {
                        // Follow the generic raw text element parsing algorithm.
                        self.generic_parsing_algorithm(tagdata, GenericParsingAlgorithm::RawText);
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening && tagdata.name == static_interned!("select") =>
                    {
                        // Reconstruct the active formatting elements, if any.
                        self.reconstruct_active_formatting_elements();

                        // Insert an HTML element for the token.
                        self.insert_html_element_for_token(&tagdata);

                        // Set the frameset-ok flag to "not ok".
                        self.frameset_ok = FramesetOkFlag::NotOk;

                        // If the insertion mode is one of "in table", "in caption", "in table body", "in row", or "in cell",
                        // then switch the insertion mode to "in select in table".
                        if self.insertion_mode == InsertionMode::InTable
                            || self.insertion_mode == InsertionMode::InCaption
                            || self.insertion_mode == InsertionMode::InTableBody
                            || self.insertion_mode == InsertionMode::InRow
                            || self.insertion_mode == InsertionMode::InCell
                        {
                            self.insertion_mode = InsertionMode::InSelectInTable;
                        } else {
                            // Otherwise, switch the insertion mode to "in select".
                            self.insertion_mode = InsertionMode::InSelect;
                        }
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening && tagdata.name == static_interned!("optgroup")
                            || tagdata.name == static_interned!("option") =>
                    {
                        // If the current node is an option element, then pop the current node off the stack of open elements.
                        if self.current_node().borrow().local_name() == static_interned!("option") {
                            self.pop_from_open_elements();
                        }

                        // Reconstruct the active formatting elements, if any.
                        self.reconstruct_active_formatting_elements();

                        // Insert an HTML element for the token.
                        self.insert_html_element_for_token(&tagdata);
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening && tagdata.name == static_interned!("rb")
                            || tagdata.name == static_interned!("rtc") =>
                    {
                        // If the stack of open elements has a ruby element in scope, then generate implied end tags.
                        // If the current node is not now a ruby element, this is a parse error.
                        if !self.is_element_in_scope(static_interned!("ruby")) {
                            self.generate_implied_end_tags()
                        }

                        // Insert an HTML element for the token.
                        self.insert_html_element_for_token(&tagdata);
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening && tagdata.name == static_interned!("rp")
                            || tagdata.name == static_interned!("rt") =>
                    {
                        // If the stack of open elements has a ruby element in scope, then generate implied end tags,
                        // except for rtc elements. If the current node is not now a rtc element or a ruby element,
                        // this is a parse error.
                        if !self.is_element_in_scope(static_interned!("ruby")) {
                            self.generate_implied_end_tags_excluding(Some(static_interned!("rtc")));
                        }

                        // Insert an HTML element for the token.
                        self.insert_html_element_for_token(&tagdata);
                    },
                    Token::Tag(mut tagdata)
                        if tagdata.opening && tagdata.name == static_interned!("math") =>
                    {
                        // Reconstruct the active formatting elements, if any.
                        self.reconstruct_active_formatting_elements();

                        // Adjust MathML attributes for the token. (This fixes the case of MathML attributes that are not all lowercase.)
                        tagdata.adjust_mathml_attributes();

                        // Adjust foreign attributes for the token. (This fixes the use of namespaced attributes, in particular XLink.)
                        tagdata.adjust_foreign_attributes();

                        // Insert a foreign element for the token, with MathML namespace and false.
                        self.insert_foreign_element(&tagdata, Namespace::MathML, false);

                        // If the token has its self-closing flag set, pop the current node off the stack of open elements
                        // and acknowledge the token's self-closing flag.
                        if tagdata.self_closing {
                            self.pop_from_open_elements();
                            self.acknowledge_self_closing_flag_if_set(&tagdata);
                        }
                    },
                    Token::Tag(mut tagdata)
                        if tagdata.opening && tagdata.name == static_interned!("svg") =>
                    {
                        // Reconstruct the active formatting elements, if any.
                        self.reconstruct_active_formatting_elements();

                        // Adjust SVG attributes for the token. (This fixes the case of SVG attributes that are not all lowercase.)
                        tagdata.adjust_svg_attributes();

                        // Adjust foreign attributes for the token. (This fixes the use of namespaced attributes, in particular XLink in SVG.)
                        tagdata.adjust_foreign_attributes();

                        // Insert a foreign element for the token, with SVG namespace and false.
                        self.insert_foreign_element(&tagdata, Namespace::SVG, false);

                        // If the token has its self-closing flag set, pop the current node off the
                        // stack of open elements and acknowledge the token's self-closing flag.
                        if tagdata.self_closing {
                            self.pop_from_open_elements();
                            self.acknowledge_self_closing_flag_if_set(&tagdata);
                        }
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening
                            && (tagdata.name == static_interned!("caption")
                                || tagdata.name == static_interned!("col")
                                || tagdata.name == static_interned!("colgroup")
                                || tagdata.name == static_interned!("frame")
                                || tagdata.name == static_interned!("head")
                                || tagdata.name == static_interned!("tbody")
                                || tagdata.name == static_interned!("td")
                                || tagdata.name == static_interned!("tfoot")
                                || tagdata.name == static_interned!("th")
                                || tagdata.name == static_interned!("thead")
                                || tagdata.name == static_interned!("tr")) =>
                    {
                        // Parse error. Ignore the token.
                    },
                    Token::Tag(tagdata) if tagdata.opening => {
                        // Reconstruct the active formatting elements, if any.
                        self.reconstruct_active_formatting_elements();

                        // Insert an HTML element for the token.
                        self.insert_html_element_for_token(&tagdata);
                    },
                    Token::Tag(tagdata) if !tagdata.opening => {
                        self.any_other_end_tag_in_body(tagdata)
                    },

                    // FIXME a lot of (for now) irrelevant rules are missing here
                    _ => todo!(),
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-incdata
            InsertionMode::Text => {
                match token {
                    Token::Character(c) => {
                        // Insert the token's character.
                        self.insert_character(c);
                    },
                    Token::EOF => {
                        // Parse error.

                        // FIXME: If the current node is a script element, then set its already started to true.

                        // Pop the current node off the stack of open elements.
                        self.pop_from_open_elements();

                        // Switch the insertion mode to the original insertion mode and reprocess the token.
                        self.switch_back_to_original_insertion_mode();

                        self.consume(token);
                    },
                    Token::Tag(ref tagdata)
                        if !tagdata.opening && tagdata.name == static_interned!("script") =>
                    {
                        log::warn!("FIXME: implement closing script tag in text mode");

                        // If the active speculative HTML parser is null and the JavaScript execution context stack is empty, then perform a microtask checkpoint.

                        // Let script be the current node (which will be a script element).
                        let script = self.current_node();
                        script
                            .try_into_type::<HtmlScriptElement>()
                            .expect("current node must be a script element");

                        // Pop the current node off the stack of open elements.
                        self.pop_from_open_elements();

                        // Switch the insertion mode to the original insertion mode.
                        self.switch_back_to_original_insertion_mode();

                        // FIXME: the rest of this method is concerned with scripting, which we don't support yet.
                    },
                    Token::Tag(ref tagdata) if !tagdata.opening => {
                        // Pop the current node off the stack of open elements.
                        self.pop_from_open_elements();

                        // Switch the insertion mode to the original insertion mode.
                        self.insertion_mode = self
                            .original_insertion_mode
                            .expect("original insertion mode has not been set");
                    },
                    _ => todo!(),
                }
            },
            // <https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-intable>
            InsertionMode::InTable => {
                match token {
                    Token::Character(_) => {
                        // FIXME: there is another condition to this branch
                        // Let the pending table character tokens be an empty list of tokens.
                        self.pending_table_character_tokens.clear();

                        // Let the original insertion mode be the current insertion mode.
                        self.original_insertion_mode = Some(self.insertion_mode);

                        // Switch the insertion mode to "in table text" and reprocess the token.
                        self.insertion_mode = InsertionMode::InTableText;
                        self.consume(token);
                    },
                    Token::Comment(comment) => {
                        // Insert a comment.
                        self.insert_comment(comment);
                    },
                    Token::DOCTYPE(_) => {
                        // Parse error. Ignore the token.
                    },
                    Token::Tag(tag) if tag.opening && tag.name == static_interned!("caption") => {
                        // Clear the stack back to a table context.
                        self.clear_the_stack_back_to_a_table_context();

                        // Insert a marker at the end of the list of active formatting elements.
                        self.active_formatting_elements.push_marker();

                        // Insert an HTML element for the token, then switch the insertion mode to "in caption".
                        self.insert_html_element_for_token(&tag);
                        self.insertion_mode = InsertionMode::InCaption;
                    },
                    Token::Tag(tag) if tag.opening && tag.name == static_interned!("colgroup") => {
                        // Clear the stack back to a table context.
                        self.clear_the_stack_back_to_a_table_context();

                        // Insert an HTML element for the token, then switch the insertion mode to "in column group".
                        self.insert_html_element_for_token(&tag);
                        self.insertion_mode = InsertionMode::InColumnGroup;
                    },
                    Token::Tag(ref tag) if tag.opening && tag.name == static_interned!("col") => {
                        // Clear the stack back to a table context.
                        self.clear_the_stack_back_to_a_table_context();

                        // Insert an HTML element for a "colgroup" start tag token with no attributes,
                        // then switch the insertion mode to "in column group".
                        let fake_tag = TagData {
                            opening: true,
                            name: static_interned!("colgroup"),
                            self_closing: false,
                            attributes: vec![],
                        };
                        self.insert_html_element_for_token(&fake_tag);
                        self.insertion_mode = InsertionMode::InColumnGroup;

                        // Reprocess the current token.
                        self.consume(token);
                    },
                    Token::Tag(tag)
                        if tag.opening
                            && matches!(
                                tag.name,
                                static_interned!("tbody")
                                    | static_interned!("tfoot")
                                    | static_interned!("thead")
                            ) =>
                    {
                        // Clear the stack back to a table context.
                        self.clear_the_stack_back_to_a_table_context();

                        // Insert an HTML element for the token, then switch the insertion mode to "in table body".
                        self.insert_html_element_for_token(&tag);
                        self.insertion_mode = InsertionMode::InTableBody;
                    },
                    Token::Tag(ref tag)
                        if tag.opening
                            && matches!(
                                tag.name,
                                static_interned!("td")
                                    | static_interned!("th")
                                    | static_interned!("tr")
                            ) =>
                    {
                        // Clear the stack back to a table context.
                        self.clear_the_stack_back_to_a_table_context();

                        // Insert an HTML element for a "tbody" start tag token with no attributes,
                        // then switch the insertion mode to "in table body".
                        let fake_tag = TagData {
                            opening: true,
                            name: static_interned!("tbody"),
                            self_closing: false,
                            attributes: vec![],
                        };
                        self.insert_html_element_for_token(&fake_tag);
                        self.insertion_mode = InsertionMode::InTableBody;

                        // Reprocess the current token.
                        self.consume(token);
                    },
                    Token::Tag(ref tag) if tag.opening && tag.name == static_interned!("table") => {
                        // Parse error.
                        // If the stack of open elements does not have a table element in table scope, ignore the token.
                        if !self.is_element_in_table_scope(static_interned!("table")) {
                            return;
                        }

                        // Otherwise:
                        // Pop elements from this stack until a table element has been popped from the stack.
                        self.pop_from_open_elements_until(|node| node.is_a::<HtmlTableElement>());

                        // Reset the insertion mode appropriately.
                        self.reset_insertion_mode_appropriately();

                        // Reprocess the token.
                        self.consume(token);
                    },
                    Token::Tag(tag) if !tag.opening && tag.name == static_interned!("table") => {
                        // If the stack of open elements does not have a table element in table scope,
                        // this is a parse error; ignore the token.
                        if !self.is_element_in_table_scope(static_interned!("table")) {
                            return;
                        }

                        // Otherwise:
                        // Pop elements from this stack until a table element has been popped from the stack.
                        self.pop_from_open_elements_until(|node| node.is_a::<HtmlTableElement>());

                        // Reset the insertion mode appropriately.
                        self.reset_insertion_mode_appropriately();
                    },
                    Token::Tag(tag)
                        if !tag.opening
                            && matches!(
                                tag.name,
                                static_interned!("body")
                                    | static_interned!("caption")
                                    | static_interned!("col")
                                    | static_interned!("colgroup")
                                    | static_interned!("html")
                                    | static_interned!("tbody")
                                    | static_interned!("td")
                                    | static_interned!("tfoot")
                                    | static_interned!("th")
                                    | static_interned!("thead")
                                    | static_interned!("tr")
                            ) =>
                    {
                        // Parse error. Ignore the token.
                    },
                    Token::Tag(ref tag)
                        if tag.opening
                            && matches!(
                                tag.name,
                                static_interned!("style")
                                    | static_interned!("script")
                                    | static_interned!("template")
                            ) =>
                    {
                        // Process the token using the rules for the "in head" insertion mode.
                        self.consume_in_mode(InsertionMode::InHead, token);
                    },
                    Token::Tag(tag) if tag.opening && tag.name == static_interned!("input") => {
                        // FIXME:
                        // If the token does not have an attribute with the name "type", or if it does,
                        // but that attribute's value is not an ASCII case-insensitive match for the string "hidden",
                        // then: act as described in the "anything else" entry below.

                        // Otherwise:
                        // Parse error.
                        // Insert an HTML element for the token.
                        self.insert_html_element_for_token(&tag);

                        // Pop that input element off the stack of open elements.
                        self.open_elements.pop();

                        // Acknowledge the token's self-closing flag, if it is set.
                        self.acknowledge_self_closing_flag_if_set(&tag);
                    },
                    Token::Tag(tag) if tag.opening && tag.name == static_interned!("form") => {
                        // Parse error.
                        // If there is a template element on the stack of open elements, or if the form element pointer is not null, ignore the token.
                        if self
                            .open_elements
                            .iter()
                            .any(|elem| elem.is_a::<HtmlTemplateElement>())
                            || self.form.is_some()
                        {
                            return;
                        }

                        // Otherwise:
                        // Insert an HTML element for the token, and set the form element pointer to point to the element created.
                        let element = self.insert_html_element_for_token(&tag);
                        let form_element = element
                            .try_into_type()
                            .expect("No form element was created");
                        self.form = Some(form_element);

                        // Pop that form element off the stack of open elements.
                        self.open_elements.pop();
                    },
                    Token::EOF => {
                        // Process the token using the rules for the "in body" insertion mode.
                        self.consume_in_mode(InsertionMode::InBody, token);
                    },
                    _ => {
                        // Parse error.
                        // Enable foster parenting, process the token using the rules for the "in body" insertion mode, and then disable foster parenting.
                        self.is_foster_parenting_enabled = true;
                        self.consume_in_mode(InsertionMode::InBody, token);
                        self.is_foster_parenting_enabled = false;
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-intabletext
            InsertionMode::InTableText => {
                match token {
                    Token::Character('\x00') => {
                        // Parse error. Ignore the token.
                    },
                    Token::Character(c) => {
                        // Append the character token to the pending table character tokens list.
                        self.pending_table_character_tokens.push(c);
                    },
                    _ => {
                        // If any of the tokens in the pending table character tokens list are character tokens
                        // that are not ASCII whitespace, then this is a parse error: reprocess the character tokens
                        // in the pending table character tokens list using the rules given in the
                        // "anything else" entry in the "in table" insertion mode.
                        if self
                            .pending_table_character_tokens
                            .iter()
                            .any(|&c| c != ' ')
                        {
                            // NOTE: This empties the list of pending table characters, which shouldn't be a problem
                            let pending_table_character_tokens =
                                mem::take(&mut self.pending_table_character_tokens);
                            for c in pending_table_character_tokens {
                                self.is_foster_parenting_enabled = true;
                                self.consume_in_mode(InsertionMode::InBody, Token::Character(c));
                                self.is_foster_parenting_enabled = false;
                            }
                        } else {
                            // Otherwise, insert the characters given by the pending table character tokens list.
                            for c in &self.pending_table_character_tokens {
                                self.insert_character(*c);
                            }
                        }

                        // Switch the insertion mode to the original insertion mode and reprocess the token.
                        self.insertion_mode = self
                            .original_insertion_mode
                            .take()
                            .expect("Original insertion mode not set");
                        self.consume(token);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-incaption
            InsertionMode::InCaption => todo!("implement InCaption mode"),

            // https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-incolgroup
            InsertionMode::InColumnGroup => todo!("implement InColumnGroup mode"),

            // https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-intbody
            InsertionMode::InTableBody => {
                match token {
                    Token::Tag(ref tag) if tag.opening && tag.name == static_interned!("tr") => {
                        // Clear the stack back to a table body context.
                        self.clear_the_stack_back_to_a_table_context();

                        // Insert an HTML element for the token, then switch the insertion mode to "in row".
                        self.insert_html_element_for_token(tag);

                        self.insertion_mode = InsertionMode::InRow;
                    },
                    Token::Tag(ref tag)
                        if tag.opening
                            && matches!(
                                tag.name,
                                static_interned!("th") | static_interned!("td")
                            ) =>
                    {
                        // Parse error.
                        // Clear the stack back to a table body context.
                        self.clear_the_stack_back_to_a_table_context();

                        // Insert an HTML element for a "tr" start tag token with no attributes, then switch the insertion mode to "in row".
                        let fake_tag = TagData {
                            opening: true,
                            name: static_interned!("tr"),
                            self_closing: false,
                            attributes: vec![],
                        };
                        self.insert_html_element_for_token(&fake_tag);
                        self.insertion_mode = InsertionMode::InRow;

                        // Reprocess the current token.
                        self.consume(token);
                    },
                    Token::Tag(ref tag)
                        if !tag.opening
                            && matches!(
                                tag.name,
                                static_interned!("tbody")
                                    | static_interned!("tfoot")
                                    | static_interned!("thead")
                            ) =>
                    {
                        // If the stack of open elements does not have an element in table scope that is an
                        // HTML element with the same tag name as the token, this is a parse error; ignore the token.
                        if !self.is_element_in_table_scope(tag.name) {
                            return;
                        }

                        // Otherwise:
                        // Clear the stack back to a table body context.
                        self.clear_the_stack_back_to_a_table_context();

                        // Pop the current node from the stack of open elements.
                        // Switch the insertion mode to "in table".
                        self.pop_from_open_elements();

                        self.insertion_mode = InsertionMode::InTable;
                    },
                    Token::Tag(ref tag)
                        if (tag.opening
                            && matches!(
                                tag.name,
                                static_interned!("caption")
                                    | static_interned!("col")
                                    | static_interned!("colgroup")
                                    | static_interned!("tbody")
                                    | static_interned!("tfoot")
                                    | static_interned!("thead")
                            ))
                            || (!tag.opening && tag.name == static_interned!("table")) =>
                    {
                        // FIXME: If the stack of open elements does not have a tbody, thead, or tfoot element in table scope,
                        // this is a parse error; ignore the token.

                        // Otherwise:
                        // Clear the stack back to a table body context.
                        self.clear_the_stack_back_to_a_table_context();

                        // Pop the current node from the stack of open elements.
                        // Switch the insertion mode to "in table".
                        self.pop_from_open_elements();
                        self.insertion_mode = InsertionMode::InTable;

                        // Reprocess the token.
                        self.consume(token);
                    },
                    Token::Tag(tag)
                        if !tag.opening
                            && matches!(
                                tag.name,
                                static_interned!("body")
                                    | static_interned!("caption")
                                    | static_interned!("col")
                                    | static_interned!("colgroup")
                                    | static_interned!("html")
                                    | static_interned!("td")
                                    | static_interned!("th")
                                    | static_interned!("tr")
                            ) =>
                    {
                        // Parse error. Ignore the token.
                    },
                    _ => {
                        // Process the token using the rules for the "in table" insertion mode.
                        self.consume_in_mode(InsertionMode::InTable, token);
                    },
                }
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-intr
            InsertionMode::InRow => todo!("implement InRow state"),

            // https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-intd
            InsertionMode::InCell => todo!("implement InCell state"),

            // https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inselect
            InsertionMode::InSelect => todo!("implement InSelect state"),

            // https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inselectintable
            InsertionMode::InSelectInTable => todo!("implement InSelectInTable mode"),

            // https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-intemplate
            InsertionMode::InTemplate => {
                match token {
                    Token::Character(_) | Token::Comment(_) | Token::DOCTYPE(_) => {
                        // Process the token using the rules for the "in body" insertion mode.
                        self.consume_in_mode(InsertionMode::InBody, token);
                    },
                    Token::Tag(ref tag)
                        if tag.opening
                            && matches!(
                                tag.name,
                                static_interned!("base")
                                    | static_interned!("basefont")
                                    | static_interned!("bgsound")
                                    | static_interned!("link")
                                    | static_interned!("meta")
                                    | static_interned!("noframes")
                                    | static_interned!("script")
                                    | static_interned!("style")
                                    | static_interned!("template")
                                    | static_interned!("title")
                            ) =>
                    {
                        // Process the token using the rules for the "in head" insertion mode.
                        self.consume_in_mode(InsertionMode::InHead, token);
                    },
                    Token::Tag(ref tag)
                        if !tag.opening && tag.name == static_interned!("template") =>
                    {
                        // Process the token using the rules for the "in head" insertion mode.
                        self.consume_in_mode(InsertionMode::InHead, token);
                    },
                    Token::Tag(ref tag)
                        if tag.opening
                            && matches!(
                                tag.name,
                                static_interned!("caption")
                                    | static_interned!("colgroup")
                                    | static_interned!("tbody")
                                    | static_interned!("tfoot")
                                    | static_interned!("thead")
                            ) =>
                    {
                        // Pop the current template insertion mode off the stack of template insertion modes.
                        self.template_insertion_modes.pop();

                        // Push "in table" onto the stack of template insertion modes so that it is the new
                        // current template insertion mode.
                        self.template_insertion_modes.push(InsertionMode::InTable);

                        // Switch the insertion mode to "in table", and reprocess the token.
                        self.insertion_mode = InsertionMode::InTable;
                        self.consume(token);
                    },
                    Token::Tag(ref tag) if tag.opening && tag.name == static_interned!("col") => {
                        // Pop the current template insertion mode off the stack of template insertion modes.
                        self.template_insertion_modes.pop();

                        // Push "in column group" onto the stack of template insertion modes so that it is
                        // the new current template insertion mode.
                        self.template_insertion_modes
                            .push(InsertionMode::InColumnGroup);

                        // Switch the insertion mode to "in column group", and reprocess the token.
                        self.insertion_mode = InsertionMode::InColumnGroup;
                        self.consume(token);
                    },
                    Token::Tag(ref tag) if tag.opening && tag.name == static_interned!("tr") => {
                        // Pop the current template insertion mode off the stack of template insertion modes.
                        self.template_insertion_modes.pop();

                        // Push "in table body" onto the stack of template insertion modes so that it is
                        // the new current template insertion mode.
                        self.template_insertion_modes
                            .push(InsertionMode::InTableBody);

                        // Switch the insertion mode to "in table body", and reprocess the token.
                        self.insertion_mode = InsertionMode::InTableBody;
                        self.consume(token);
                    },
                    Token::Tag(ref tag)
                        if tag.opening
                            && matches!(
                                tag.name,
                                static_interned!("td") | static_interned!("th")
                            ) =>
                    {
                        // Pop the current template insertion mode off the stack of template insertion modes.
                        self.template_insertion_modes.pop();

                        // Push "in row" onto the stack of template insertion modes so that it is the new current template insertion mode.
                        self.template_insertion_modes.push(InsertionMode::InRow);

                        // Switch the insertion mode to "in row", and reprocess the token.
                        self.insertion_mode = InsertionMode::InRow;
                        self.consume(token);
                    },
                    Token::Tag(ref tag) if tag.opening => {
                        // Pop the current template insertion mode off the stack of template insertion modes.
                        self.template_insertion_modes.pop();

                        // Push "in body" onto the stack of template insertion modes so that it is the new current template insertion mode.
                        self.template_insertion_modes.push(InsertionMode::InBody);

                        // Switch the insertion mode to "in body", and reprocess the token.
                        self.insertion_mode = InsertionMode::InBody;
                        self.consume(token);
                    },
                    Token::Tag(_) => {
                        // Parse error. Ignore the token.
                    },
                    Token::EOF => {
                        // If there is no template element on the stack of open elements, then stop parsing. (fragment case)
                        let contains_template_element = self
                            .open_elements
                            .iter()
                            .any(|element| element.is_a::<HtmlTemplateElement>());
                        if !contains_template_element {
                            self.stop_parsing();
                        } else {
                            // Otherwise, this is a parse error.
                            // Pop elements from the stack of open elements until a template element has been popped from the stack.
                            self.pop_from_open_elements_until(|elem| {
                                elem.borrow().local_name() == static_interned!("template")
                            });

                            // Clear the list of active formatting elements up to the last marker.
                            self.active_formatting_elements.clear_up_to_last_marker();

                            // Pop the current template insertion mode off the stack of template insertion modes.
                            self.template_insertion_modes.pop();

                            // Reset the insertion mode appropriately.
                            self.reset_insertion_mode_appropriately();

                            // Reprocess the token.
                            self.consume(token);
                        }
                    },
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
                    Token::Tag(ref tagdata)
                        if tagdata.opening && tagdata.name == static_interned!("html") =>
                    {
                        // Process the token using the rules for the "in body" insertion mode.
                        self.consume_in_mode(InsertionMode::InBody, token);
                    },
                    Token::Tag(tagdata)
                        if !tagdata.opening && tagdata.name == static_interned!("html") =>
                    {
                        // FIXME
                        // If the parser was created as part of the HTML fragment parsing
                        // algorithm, this is a parse error; ignore the token. (fragment case)

                        // Otherwise, switch the insertion mode to "after after body".
                        self.insertion_mode = InsertionMode::AfterAfterBody;
                    },
                    Token::EOF => {
                        // Stop parsing.
                        self.stop_parsing();
                    },
                    _ => {
                        // Parse error. Switch the insertion mode to "in body" and reprocess the
                        // token.
                        self.insertion_mode = InsertionMode::InBody;
                        self.consume(token);
                    },
                }
            },
            // https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inframeset
            InsertionMode::InFrameset => todo!("implement InFrameset mode"),

            // https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-afterframeset
            InsertionMode::AfterFrameset => todo!("implement AfterFrameset mode"),

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
                    Token::Tag(ref tagdata)
                        if tagdata.opening && tagdata.name == static_interned!("html") =>
                    {
                        // Process the token using the rules for the "in body" insertion mode.
                        self.consume_in_mode(InsertionMode::InBody, token);
                    },
                    Token::EOF => {
                        // Stop parsing.
                        self.stop_parsing();
                    },
                    _ => {
                        // Parse error. Switch the insertion mode to "in body" and reprocess the
                        // token.
                        self.insertion_mode = InsertionMode::InBody;
                        self.consume(token);
                    },
                }
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#the-after-after-frameset-insertion-mode
            InsertionMode::AfterAfterFrameset => todo!("implement AfterAfterFrameset mode"),
        }
    }

    /// Extracted into its own functions because the adoption agency algorithm makes use of this sequence
    /// of steps too.
    fn any_other_end_tag_in_body(&mut self, tag: TagData) {
        fn is_html_element_with_name(node: &DomPtr<Element>, name: InternedString) -> bool {
            if let Some(element) = node.try_into_type::<Element>() {
                if element.borrow().local_name() == name {
                    return node.is_a::<HtmlElement>();
                }
            }
            false
        }
        // Run these steps:

        // 1. Initialize node to be the current node (the bottommost node of the stack).
        // 2. Loop:
        for (index, node) in self.open_elements.iter().rev().enumerate() {
            // If node is an HTML element with the same tag name as the token, then:
            if is_html_element_with_name(&node, tag.name) {
                // 1. Generate implied end tags,
                // FIXME: except for HTML elements with the same tag name as the token.
                self.generate_implied_end_tags_excluding(None);

                // 2. If node is not the current node,
                if index != 0 {
                    // then this is a parse error
                }

                // 3. Pop all the nodes from the current node up to node, including node, then stop these steps.
                for _ in 0..index + 1 {
                    self.open_elements.pop();
                }
                break;
            }
            // 3. Otherwise, if node is in the special category,
            else if is_element_in_special_category(node.borrow().local_name()) {
                // then this is a parse error; ignore the token, and return.
                return;
            }

            // NOTE: Steps 4 & 5 are implemented using the for-loop
            // 4. Set node to the previous entry in the stack of open elements.
            // 5. Return to the step labeled loop.
        }
    }

    fn switch_back_to_original_insertion_mode(&mut self) {
        self.insertion_mode = self
            .original_insertion_mode
            .take()
            .expect("Original insertion mode has not been set");
    }
}

/// <https://html.spec.whatwg.org/multipage/parsing.html#special>
fn is_element_in_special_category(tagname: InternedString) -> bool {
    matches!(
        tagname,
        static_interned!("address")
            | static_interned!("applet")
            | static_interned!("area")
            | static_interned!("article")
            | static_interned!("aside")
            | static_interned!("base")
            | static_interned!("basefont")
            | static_interned!("bgsound")
            | static_interned!("blockquote")
            | static_interned!("body")
            | static_interned!("br")
            | static_interned!("button")
            | static_interned!("caption")
            | static_interned!("center")
            | static_interned!("col")
            | static_interned!("colgroup")
            | static_interned!("dd")
            | static_interned!("details")
            | static_interned!("dir")
            | static_interned!("div")
            | static_interned!("dl")
            | static_interned!("dt")
            | static_interned!("embed")
            | static_interned!("fieldset")
            | static_interned!("figcaption")
            | static_interned!("figure")
            | static_interned!("footer")
            | static_interned!("form")
            | static_interned!("frame")
            | static_interned!("frameset")
            | static_interned!("h1")
            | static_interned!("h2")
            | static_interned!("h3")
            | static_interned!("h4")
            | static_interned!("h5")
            | static_interned!("h6")
            | static_interned!("head")
            | static_interned!("header")
            | static_interned!("hgroup")
            | static_interned!("hr")
            | static_interned!("html")
            | static_interned!("iframe")
            | static_interned!("img")
            | static_interned!("input")
            | static_interned!("keygen")
            | static_interned!("li")
            | static_interned!("link")
            | static_interned!("listing")
            | static_interned!("main")
            | static_interned!("marquee")
            | static_interned!("menu")
            | static_interned!("meta")
            | static_interned!("nav")
            | static_interned!("noembed")
            | static_interned!("noframes")
            | static_interned!("noscript")
            | static_interned!("object")
            | static_interned!("ol")
            | static_interned!("p")
            | static_interned!("param")
            | static_interned!("plaintext")
            | static_interned!("pre")
            | static_interned!("script")
            | static_interned!("search")
            | static_interned!("section")
            | static_interned!("select")
            | static_interned!("source")
            | static_interned!("style")
            | static_interned!("summary")
            | static_interned!("table")
            | static_interned!("tbody")
            | static_interned!("td")
            | static_interned!("template")
            | static_interned!("tfoot")
            | static_interned!("th")
            | static_interned!("thead")
            | static_interned!("title")
            | static_interned!("tr")
            | static_interned!("track")
            | static_interned!("ul")
            | static_interned!("wbr")
            | static_interned!("xmp")
            | static_interned!("mi")
            | static_interned!("mo")
            | static_interned!("mn")
            | static_interned!("ms")
            | static_interned!("mtext")
            | static_interned!("annotation-xml")
            | static_interned!("foreignObject")
            | static_interned!("desc")
    )
}
