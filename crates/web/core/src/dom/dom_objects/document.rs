use dom_derive::inherit;

use crate::dom::DomPtr;

use super::{HtmlScriptElement, Node};

/// <https://dom.spec.whatwg.org/#interface-document>
#[inherit(Node)]
pub struct Document {
    charset: String,

    /// <https://html.spec.whatwg.org/multipage/scripting.html#list-of-scripts-that-will-execute-when-the-document-has-finished-parsing>
    script_that_will_execute_when_the_document_has_finished_parsing: Vec<DomPtr<HtmlScriptElement>>,
}

impl Document {
    /// Queue a script to be executed to be executed
    /// [once the document has finished parsing](https://html.spec.whatwg.org/multipage/scripting.html#list-of-scripts-that-will-execute-when-the-document-has-finished-parsing)
    pub fn execute_script_when_document_has_finished_parsing(
        &mut self,
        script: DomPtr<HtmlScriptElement>,
    ) {
        self.script_that_will_execute_when_the_document_has_finished_parsing
            .push(script);
    }

    #[must_use]
    pub fn pop_next_script_to_execute_when_the_document_has_finished_parsing(
        &mut self,
    ) -> Option<DomPtr<HtmlScriptElement>> {
        self.script_that_will_execute_when_the_document_has_finished_parsing
            .pop()
    }

    #[must_use]
    pub fn charset(&self) -> &str {
        &self.charset
    }

    /// <https://html.spec.whatwg.org/multipage/semantics.html#has-a-style-sheet-that-is-blocking-scripts>
    #[must_use]
    pub fn has_a_style_sheet_that_is_blocking_scripts(&self) -> bool {
        // FIXME: Implement this

        // 5. Return false.
        false
    }

    /// <https://html.spec.whatwg.org/multipage/document-sequences.html#fully-active>
    #[must_use]
    pub fn is_fully_active(&self) -> bool {
        // FIXME: Implement this
        true
    }
}
