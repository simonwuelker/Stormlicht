//! <https://dom.spec.whatwg.org/>

mod codegen;
pub mod dom_objects;
mod dom_ptr;

pub use codegen::{DOMType, DOMTyped};
pub use dom_ptr::{DOMPtr, WeakDOMPtr};

use self::dom_objects::{Document, Element};
use crate::infra::Namespace;

/// <https://dom.spec.whatwg.org/#concept-element-custom-element-state>
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum ElementCustomState {
    Undefined,
    Failed,
    #[default]
    Uncustomized,
    Precustomized,
    Custom,
}

/// <https://dom.spec.whatwg.org/#concept-create-element>
pub fn create_element(
    document: WeakDOMPtr<Document>,
    local_name: String,
    namespace: Namespace,
    prefix: Option<String>,
    is: Option<String>,
    _synchronous_custom_elements_flag: bool,
) -> DOMPtr<Element> {
    // FIXME: make this spec-compliant!

    let mut element = Element::new(
        namespace,
        prefix,
        local_name,
        ElementCustomState::Uncustomized,
        is,
    );
    element.set_owning_document(document);

    DOMPtr::new(element)
}

/// <https://html.spec.whatwg.org/multipage/custom-elements.html#look-up-a-custom-element-definition>
pub fn lookup_custom_element_definition(
    namespace: Namespace,
    _local_name: &str,
    _is: Option<&str>,
) -> Option<()> {
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
