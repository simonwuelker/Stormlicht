//! Implements the [Tree Construction Stage](https://html.spec.whatwg.org/multipage/parsing.html#tree-construction)

use crate::{
    css::{self, Stylesheet},
    dom::{
        self,
        dom_objects::{
            Comment, Document, DocumentType, Element, HtmlBodyElement, HtmlDdElement,
            HtmlDivElement, HtmlElement, HtmlFormElement, HtmlHeadElement, HtmlHtmlElement,
            HtmlLiElement, HtmlParagraphElement, HtmlScriptElement, HtmlTemplateElement, Node,
            Text,
        },
        DOMPtr, DOMType, DOMTyped,
    },
    html::{
        tokenization::{ParseErrorHandler, TagData, Token, Tokenizer, TokenizerState},
        treebuilding::ActiveFormattingElements,
    },
    infra::Namespace,
};

use string_interner::{static_interned, static_str, InternedString};

use super::active_formatting_elements::{ActiveFormattingElement, FormatEntry};

const TAB: char = '\u{0009}';
const LINE_FEED: char = '\u{000A}';
const FORM_FEED: char = '\u{000C}';
const WHITESPACE: char = '\u{0020}';

const DEFAULT_SCOPE: &[DOMType] = &[DOMType::HtmlHtmlElement, DOMType::HtmlTemplateElement];

/// <https://html.spec.whatwg.org/multipage/parsing.html#has-an-element-in-button-scope>
const BUTTON_SCOPE: &[DOMType] = &[
    DOMType::HtmlHtmlElement,
    DOMType::HtmlTemplateElement,
    DOMType::HtmlButtonElement,
];

/// <https://html.spec.whatwg.org/multipage/parsing.html#has-an-element-in-list-item-scope>
const ITEM_SCOPE: &[DOMType] = &[
    DOMType::HtmlHtmlElement,
    DOMType::HtmlTemplateElement,
    // <ol>
    // <ul>
];

#[derive(Clone, Copy, Debug)]
enum GenericParsingAlgorithm {
    RCDATA,
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
    document: DOMPtr<Document>,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#original-insertion-mode>
    original_insertion_mode: Option<InsertionMode>,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#stack-of-template-insertion-modes>
    template_insertion_modes: Vec<InsertionMode>,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#insertion-mode>
    insertion_mode: InsertionMode,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#stack-of-open-elements>
    open_elements: Vec<DOMPtr<Node>>,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#head-element-pointer>
    head: Option<DOMPtr<Node>>,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#frameset-ok-flag>
    frameset_ok: FramesetOkFlag,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#form-element-pointer>
    form: Option<DOMPtr<HtmlFormElement>>,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#list-of-active-formatting-elements>
    active_formatting_elements: ActiveFormattingElements,

    /// <https://html.spec.whatwg.org/multipage/parsing.html#scripting-flag>
    execute_script: bool,

    done: bool,

    stylesheets: Vec<Stylesheet>,
}

impl<P: ParseErrorHandler> Parser<P> {
    pub fn new(source: &str) -> Self {
        let document = DOMPtr::new(Document::default());
        // TODO: judging from the spec behaviour, it appears that document's document's
        // point to themselves. We should find a note for that somewhere in a spec.
        document
            .borrow_mut()
            .set_owning_document(DOMPtr::clone(&document).downgrade());

        Self {
            tokenizer: Tokenizer::new(source),
            document: document,
            original_insertion_mode: None,
            template_insertion_modes: vec![],
            insertion_mode: InsertionMode::Initial,
            open_elements: vec![],
            head: None,
            form: None,
            frameset_ok: FramesetOkFlag::default(),
            active_formatting_elements: ActiveFormattingElements::default(),
            execute_script: false,
            done: false,
            stylesheets: vec![Stylesheet::user_agent_rules()],
        }
    }

    #[must_use]
    fn open_elements_top_node(&self) -> Option<DOMPtr<Node>> {
        self.open_elements.first().cloned()
    }

    #[must_use]
    fn open_elements_bottommost_node(&self) -> Option<DOMPtr<Node>> {
        self.open_elements.last().cloned()
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#acknowledge-self-closing-flag>
    #[inline]
    fn acknowledge_self_closing_flag_if_set(&self, _tag: &TagData) {
        // This is only relevant for detecting parse errors, which we currently don't care about
    }

    #[must_use]
    fn find_in_open_elements<T: DOMTyped>(&self, needle: &DOMPtr<T>) -> Option<usize> {
        self.open_elements
            .iter()
            .enumerate()
            .find(|(_, node)| node.ptr_eq(needle))
            .map(|(i, _)| i)
    }

    /// Pop elements from the stack of open elements until a element matching a predicate has been popped
    fn pop_from_open_elements_until<F: Fn(DOMPtr<Node>) -> bool>(&mut self, predicate: F) {
        while self
            .pop_from_open_elements()
            .is_some_and(|element| !predicate(element))
        {}
    }

    fn remove_from_open_elements<T: DOMTyped>(&mut self, to_remove: &DOMPtr<T>) {
        self.open_elements
            .retain_mut(|element| !DOMPtr::ptr_eq(to_remove, element))
    }

    fn pop_from_open_elements(&mut self) -> Option<DOMPtr<Node>> {
        let popped_node = self.open_elements.pop();

        // Check if we just popped a <style> element, if so, register a new stylesheet
        if let Some(node) = popped_node.as_ref() {
            if node.underlying_type() == DOMType::HtmlStyleElement {
                if let Some(first_child) = node.borrow().children().first() {
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
        }
        popped_node
    }

    pub fn parse(mut self) -> (DOMPtr<Node>, Vec<Stylesheet>) {
        while let Some(token) = self.tokenizer.next() {
            self.consume(token);

            if self.done {
                break;
            }
        }

        (
            self.open_elements_top_node()
                .expect("HTML parser did not produce a root node"),
            self.stylesheets,
        )
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#current-node>
    fn current_node(&self) -> DOMPtr<Node> {
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
        let new_node = DOMPtr::new(new_text).upcast();
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
        let new_node = DOMPtr::new(new_comment).upcast();
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
            GenericParsingAlgorithm::RCDATA => self.tokenizer.switch_to(TokenizerState::RCDATA),
        }

        // Let the original insertion mode be the current insertion mode.
        self.original_insertion_mode = Some(self.insertion_mode);

        // Then, switch the insertion mode to "text".
        self.insertion_mode = InsertionMode::Text;
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#has-an-element-in-scope>
    fn is_element_in_scope(&self, target_node_type: DOMType) -> bool {
        // FIXME: this default scope should contain more types but they dont exist yet
        self.is_element_in_specific_scope(target_node_type, DEFAULT_SCOPE)
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#has-an-element-in-button-scope>
    fn is_element_in_button_scope(&self, target_node_type: DOMType) -> bool {
        self.is_element_in_specific_scope(target_node_type, BUTTON_SCOPE)
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#has-an-element-in-the-specific-scope>
    fn is_element_in_specific_scope(&self, target_node_type: DOMType, scope: &[DOMType]) -> bool {
        // 1. Initialize node to be the current node (the bottommost node of the stack).
        let mut node = self.current_node();

        loop {
            // 2. If node is the target node, terminate in a match state.
            if node.underlying_type() == target_node_type {
                return true;
            }
            // 3. Otherwise, if node is one of the element types in list, terminate in a failure state.
            else if scope.contains(&node.underlying_type()) {
                return false;
            }

            // Otherwise, set node to the previous entry in the stack of open elements and return to step 2.
            let next_node = node
                .borrow()
                .parent_node()
                .expect("Algorithm should terminate before top of the stack is reached");
            node = next_node;
            // (This will never fail, since the loop will always terminate in the previous step if the
            // top of the stack — an html element — is reached.)
        }
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#close-a-p-element>
    fn close_p_element(&mut self) {
        // 1. Generate implied end tags, except for p elements.
        self.generate_implied_end_tags_excluding(Some(DOMType::HtmlParagraphElement));

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
    fn generate_implied_end_tags_excluding(&mut self, exclude: Option<DOMType>) {
        loop {
            let node_type = self.current_node().underlying_type();
            if exclude.is_some_and(|to_exclude| to_exclude == node_type) {
                return;
            }
            // FIXME: There are more elements here that aren't yet implemented
            if node_type != DOMType::HtmlParagraphElement {
                return;
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
    fn create_html_element_for_token(
        &self,
        tagdata: &TagData,
        namespace: Namespace,
        intended_parent: &DOMPtr<Node>,
    ) -> DOMPtr<Node> {
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
        element.upcast()
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#insert-an-html-element>
    fn insert_html_element_for_token(&mut self, tagdata: &TagData) -> DOMPtr<Node> {
        self.insert_foreign_element(tagdata, Namespace::HTML)
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#insert-a-foreign-element>
    fn insert_foreign_element(&mut self, tagdata: &TagData, namespace: Namespace) -> DOMPtr<Node> {
        // Let the adjusted insertion location be the appropriate place for inserting a node.
        let adjusted_insertion_location = self.appropriate_place_for_inserting_node();

        // Let element be the result of creating an element for the token in the given namespace, with the intended parent being the element in which the adjusted insertion location finds itself.
        let element =
            self.create_html_element_for_token(tagdata, namespace, &adjusted_insertion_location);

        // If it is possible to insert element at the adjusted insertion location, then:
        // FIXME: it is currently always possible to insert more elements

        // FIXME: If the parser was not created as part of the HTML fragment parsing algorithm, then push a new element queue onto element's relevant agent's custom element reactions stack.

        // Insert element at the adjusted insertion location.
        Node::append_child(adjusted_insertion_location, element.clone());

        // FIXME: If the parser was not created as part of the HTML fragment parsing algorithm, then pop the element queue from
        // element's relevant agent's custom element reactions stack, and invoke custom element reactions in that queue.

        // Push element onto the stack of open elements so that it is the new current node.
        self.open_elements.push(element.clone());

        // Return element.
        element
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#reconstruct-the-active-formatting-elements>
    fn reconstruct_active_formatting_elements(&mut self) {
        let is_marker_or_in_open_elements = |entry: &FormatEntry| match entry {
            FormatEntry::Element(format_element)
                if !self
                    .open_elements
                    .iter()
                    .any(|e| DOMPtr::ptr_eq(e, &format_element.element)) =>
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
                    element: new_element.try_into_type().unwrap(),
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
                        DOMPtr::ptr_eq(&formatting_element.element, &current_node)
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
            if !self.is_element_in_scope(formatting_element.element.underlying_type()) {
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
                .find(|(_, element)| is_element_in_special_category(element.borrow().local_name()))
                .map(|(i, element)| (i, element.upcast()));

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
                        if DOMPtr::ptr_eq(&node, &formatting_element.element) {
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
                        let new_element = self.create_html_element_for_token(
                            &tag,
                            Namespace::HTML,
                            &common_ancestor,
                        );
                        self.open_elements[node_index] = new_element.clone();
                        node = new_element;

                        // 7. FIXME: If last node is furthest block, then move the aforementioned bookmark to be immediately after the
                        //    new node in the list of active formatting elements.

                        // 8. Append last node to node.
                        Node::append_child(node.clone(), last_node);

                        // 9. Set last node to node.
                        last_node = node;
                    }

                    // 14. Insert whatever last node ended up being in the previous step at the appropriate place for inserting a node,
                    //     but using common ancestor as the override target.
                    let appropriate_place = self
                        .appropriate_place_for_inserting_node_with_override(Some(common_ancestor));
                    Node::append_child(appropriate_place, last_node);

                    // 15. Create an element for the token for which formatting element was created,
                    //     in the HTML namespace, with furthest block as the intended parent.
                    let new_element = self.create_html_element_for_token(
                        &formatting_element.tag,
                        Namespace::HTML,
                        &furthest_block.clone(),
                    );

                    // 16. Take all of the child nodes of furthest block and append them to the element created in the last step.
                    for child in furthest_block.borrow().children() {
                        Node::append_child(new_element.clone(), child.clone());
                    }

                    // 17. Append that new element to furthest block.
                    Node::append_child(furthest_block, new_element);

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
                .map(DOMPtr::underlying_type)
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
                        let new_node = DOMPtr::new(doctype_node).upcast();
                        Node::append_child(DOMPtr::clone(&self.document).upcast(), new_node);
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
                        let element = self.create_html_element_for_token(
                            tagdata,
                            Namespace::HTML,
                            &DOMPtr::clone(&self.document).upcast(),
                        );

                        // Append it to the Document object.
                        Node::append_child(
                            DOMPtr::clone(&self.document).upcast(),
                            DOMPtr::clone(&element),
                        );

                        // Put this element in the stack of open elements.
                        self.open_elements.push(element);

                        // Switch the insertion mode to "before head".
                        self.insertion_mode = InsertionMode::BeforeHead;
                    },
                    other => {
                        // Create an html element whose node document is the Document object.
                        let mut html_element = HtmlHtmlElement::default();
                        html_element.set_owning_document(DOMPtr::clone(&self.document).downgrade());
                        let new_node = DOMPtr::new(html_element).upcast();

                        // Append it to the Document object.
                        Node::append_child(
                            DOMPtr::clone(&self.document).upcast(),
                            DOMPtr::clone(&new_node),
                        );

                        // Put this element in the stack of open elements.
                        self.open_elements.push(new_node);

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
                        let head = self.insert_html_element_for_token(tagdata);

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
                        let head_element = self.insert_html_element_for_token(&bogus_head_token);

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
                        self.generic_parsing_algorithm(tagdata, GenericParsingAlgorithm::RCDATA);
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
                        let element = self.create_html_element_for_token(
                            tagdata,
                            Namespace::HTML,
                            &adjusted_insert_location,
                        );

                        // 3. Set the element's parser document to the Document, and set the element's force async to false.
                        log::warn!("FIXME: Set script element attributes");

                        // 4. If the parser was created as part of the HTML fragment parsing algorithm, then set the script element's already started to true. (fragment case)

                        // 5. If the parser was invoked via the document.write() or document.writeln() methods, then optionally set the script element's already started to true.

                        // 6. Insert the newly created element at the adjusted insertion location.
                        Node::append_child(adjusted_insert_location, element.clone());

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
                        assert_eq!(
                            current_node.as_ref().map(DOMPtr::underlying_type),
                            Some(DOMType::HtmlHeadElement)
                        );

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
                        debug_assert_eq!(
                            popped_node.map(|n| n.underlying_type()),
                            Some(DOMType::HtmlHeadElement)
                        );

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
                            popped_node.map(|n| n.underlying_type()),
                            Some(DOMType::HtmlNoscriptElement)
                        );
                        debug_assert_eq!(
                            self.current_node().underlying_type(),
                            DOMType::HtmlHeadElement
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
                            popped_node.map(|n| n.underlying_type()),
                            Some(DOMType::HtmlNoscriptElement)
                        );
                        debug_assert_eq!(
                            self.current_node().underlying_type(),
                            DOMType::HtmlHeadElement
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
                        let head = DOMPtr::clone(
                            self.head
                                .as_ref()
                                .expect("Spec: self.head cannot be none at this point"),
                        );
                        self.open_elements.push(DOMPtr::clone(&head));

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
                        if self.current_node().underlying_type() != DOMType::HtmlTemplateElement {
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
                        let previous_body: DOMPtr<Element> = match self.open_elements.get(2) {
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
                            self.done = true;
                        }
                    },
                    Token::Tag(ref tagdata)
                        if !tagdata.opening && tagdata.name == static_interned!("body") =>
                    {
                        // If the stack of open elements does not have a body element in scope, this is a parse error; ignore the token.
                        if !self.is_element_in_scope(DOMType::HtmlBodyElement) {
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
                        if self.is_element_in_scope(DOMType::HtmlBodyElement) {
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
                        if self.is_element_in_button_scope(DOMType::HtmlParagraphElement) {
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
                        if self.is_element_in_button_scope(DOMType::HtmlParagraphElement) {
                            self.close_p_element();
                        }

                        // If the current node is an HTML element whose tag name is one of
                        // "h1", "h2", "h3", "h4", "h5", or "h6", then this is a parse error;
                        // pop the current node off the stack of open elements.
                        if let Some(element) = self.current_node().try_into_type::<Element>() {
                            let tag_name = element.borrow().local_name();
                            if tag_name == static_interned!("h1")
                                || tag_name == static_interned!("h2")
                                || tag_name == static_interned!("h3")
                                || tag_name == static_interned!("h4")
                                || tag_name == static_interned!("h5")
                                || tag_name == static_interned!("h6")
                            {
                                self.pop_from_open_elements();
                            }
                        }

                        self.insert_html_element_for_token(&tagdata);
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening
                            && (tagdata.name == static_interned!("pre")
                                || tagdata.name == static_interned!("listing")) =>
                    {
                        todo!()
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
                            if self.is_element_in_button_scope(DOMType::HtmlParagraphElement) {
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
                            let local_name = node
                                .clone()
                                .try_into_type::<Element>()
                                .unwrap()
                                .borrow()
                                .local_name();

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
                        if self.is_element_in_button_scope(DOMType::HtmlParagraphElement) {
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
                            self.generate_implied_end_tags_excluding(Some(DOMType::HtmlDdElement));

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
                        if self.is_element_in_scope(DOMType::HtmlParagraphElement) {
                            self.close_p_element();
                        }

                        // 8. Finally, insert an HTML element for the token.
                        self.insert_html_element_for_token(&tagdata);
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening && tagdata.name == static_interned!("plaintext") =>
                    {
                        todo!()
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening && tagdata.name == static_interned!("button") =>
                    {
                        todo!()
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
                        todo!()
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
                                Some(node) if self.is_element_in_scope(node.underlying_type()) => {
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
                            if !self.is_element_in_scope(DOMType::HtmlFormElement) {
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
                        if !self.is_element_in_button_scope(DOMType::HtmlParagraphElement) {
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
                        if !self.is_element_in_specific_scope(DOMType::HtmlLiElement, ITEM_SCOPE) {
                            return;
                        }

                        // Otherwise, run these steps:
                        // 1. Generate implied end tags, except for li elements.
                        self.generate_implied_end_tags_excluding(Some(DOMType::HtmlLiElement));

                        // 2. If the current node is not an li element, then this is a parse error.

                        // 3. Pop elements from the stack of open elements until an li element has been popped from the stack.
                        self.pop_from_open_elements_until(|node| node.is_a::<HtmlLiElement>());
                    },
                    Token::Tag(ref tagdata)
                        if !tagdata.opening
                            && (tagdata.name == static_interned!("dd")
                                || tagdata.name == static_interned!("dt")) =>
                    {
                        todo!("handle dd/dt closing tag")
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

                        // Otherwise, run these steps:

                        // 1. Generate implied end tags.
                        self.generate_implied_end_tags();

                        // 2. FIXME: If the current node is not an HTML element with the same tag name as that of the token,
                        //           then this is a parse error.

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
                                element.underlying_type() == DOMType::HtmlAnchorElement
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
                        self.active_formatting_elements
                            .push(element.try_into_type::<Element>().unwrap(), tagdata);
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
                        let element = self
                            .insert_html_element_for_token(&tagdata)
                            .try_into_type::<Element>()
                            .unwrap();

                        // Push onto the list of active formatting elements that element.
                        self.active_formatting_elements.push(element, tagdata);
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening && tagdata.name == static_interned!("nobr") =>
                    {
                        // Reconstruct the active formatting elements, if any.
                        self.reconstruct_active_formatting_elements();

                        // FIXME: If the stack of open elements has a nobr element in scope, then this is a parse error; run the adoption agency algorithm for the token,
                        // then once again reconstruct the active formatting elements, if any.
                        log::warn!("FIXME: check if <nobr> is in scope");

                        // Insert an HTML element for the token
                        let element = self
                            .insert_html_element_for_token(&tagdata)
                            .try_into_type::<Element>()
                            .unwrap();

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
                        todo!();
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
                        if self.is_element_in_button_scope(DOMType::HtmlParagraphElement) {
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
                        if self.is_element_in_button_scope(DOMType::HtmlParagraphElement) {
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
                        todo!();
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening && tagdata.name == static_interned!("rb")
                            || tagdata.name == static_interned!("rtc") =>
                    {
                        todo!();
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening && tagdata.name == static_interned!("math") =>
                    {
                        todo!();
                    },
                    Token::Tag(tagdata)
                        if tagdata.opening && tagdata.name == static_interned!("svg") =>
                    {
                        todo!();
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
                    Token::Tag(ref tagdata)
                        if tagdata.opening && tagdata.name == static_interned!("html") =>
                    {
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
                        self.insertion_mode = self.original_insertion_mode.unwrap();
                    },
                    _ => todo!(),
                }
            },
            _ => todo!("Implement '{mode:?}' state"),
        }
    }

    /// Extracted into its own functions because the adoption agency algorithm makes use of this sequence
    /// of steps too.
    fn any_other_end_tag_in_body(&mut self, tag: TagData) {
        fn is_html_element_with_name(node: DOMPtr<Node>, name: InternedString) -> bool {
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
        for node in self.open_elements.iter().rev() {
            // If node is an HTML element with the same tag name as the token, then:
            if is_html_element_with_name(node.clone(), tag.name) {
                // 1. Generate implied end tags,
                // FIXME: except for HTML elements with the same tag name as the token.
                self.generate_implied_end_tags_excluding(None);

                // 2. FIXME: If node is not the current node, then this is a parse error.

                // 3. FIXME: Pop all the nodes from the current node up to node, including node, then stop these steps.
                self.open_elements.pop();
                break;
            }
            // 3. FIXME: Otherwise, if node is in the special category, then this is a parse error;
            //    ignore the token, and return.

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

/// </// <https://html.spec.whatwg.org/multipage/parsing.html#special>
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
            | static_interned!("title")
    )
}
