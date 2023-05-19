//! <https://dom.spec.whatwg.org/>

mod codegen;
mod dom_display;
pub mod dom_objects;
mod dom_ptr;

pub use codegen::{DOMType, DOMTyped};
pub use dom_display::DOMDisplay;
pub use dom_ptr::{DOMPtr, WeakDOMPtr};

use crate::infra::Namespace;
use dom_objects::{
    Document, Element, HTMLBodyElement, HTMLElement, HTMLHeadElement, HTMLHtmlElement,
    HTMLNoscriptElement, HTMLScriptElement, HTMLTemplateElement,
};

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
#[allow(clippy::let_and_return)]
pub fn create_element(
    document: WeakDOMPtr<Document>,
    local_name: String,
    namespace: Namespace,
    prefix: Option<String>,
    is: Option<String>,
    _synchronous_custom_elements_flag: bool,
) -> DOMPtr<Element> {
    // FIXME: make this spec-compliant!

    // 1. If prefix was not given, let prefix be null.

    // 2. If is was not given, let is be null.

    // 3. Let result be null.
    // NOTE: the part of this algorithm that is implemented does not require a result
    // let mut result = None;

    // 4. Let definition be the result of looking up a custom element definition given document, namespace, localName, and is.
    let definition = lookup_custom_element_definition(namespace, &local_name, is.as_deref());

    let result = match definition {
        // 5. FIXME: If definition is non-null, and definition’s name is not equal to its local name (i.e., definition represents a customized built-in element), then:
        // [...]

        // 6. FIXME: Otherwise, if definition is non-null, then:
        Some(_) => {
            todo!()
        },

        // 7. Otherwise:
        None => {
            // 1. Let interface be the element interface for localName and namespace.

            // 2. Set result to a new element that implements interface, with no attributes,
            // namespace set to namespace, namespace prefix set to prefix,
            // local name set to localName, custom element state set to "uncustomized",
            // custom element definition set to null, is value set to is, and node document set to document.
            let mut element_data = Element::new(
                namespace,
                prefix,
                local_name.clone(),
                ElementCustomState::Uncustomized,
                None,
                is,
            );
            element_data.set_owning_document(document);

            let created_element =
                create_element_for_interface(&local_name, namespace, element_data);

            // 3. FIXME: If namespace is the HTML namespace, and either localName is a valid custom element name or is is non-null, then set result’s custom element state to "undefined".
            created_element
        },
    };

    // 8. Return result.
    result
}

/// <https://html.spec.whatwg.org/multipage/custom-elements.html#look-up-a-custom-element-definition>
pub fn lookup_custom_element_definition(
    namespace: Namespace,
    _local_name: &str,
    _is: Option<&str>,
) -> Option<()> {
    // 1. If namespace is not the HTML namespace, return null.
    if namespace != Namespace::HTML {
        return None;
    }

    // FIXME: If document's browsing context is null, return null.

    // FIXME: Let registry be document's relevant global object's CustomElementRegistry object.

    // FIXME: If there is custom element definition in registry with name and local name both equal to localName, return that custom element definition.

    // FIXME: If there is a custom element definition in registry with name equal to is and local name equal to localName, return that custom element definition.

    // Return null.
    None
}

fn create_element_for_interface(
    local_name: &str,
    namespace: Namespace,
    element_data: Element,
) -> DOMPtr<Element> {
    if namespace != Namespace::HTML {
        todo!();
    }

    match local_name {
        "head" => {
            DOMPtr::new(HTMLHeadElement::new(HTMLElement::new(element_data))).into_type::<Element>()
        },
        "html" => {
            DOMPtr::new(HTMLHtmlElement::new(HTMLElement::new(element_data))).into_type::<Element>()
        },
        "body" => {
            DOMPtr::new(HTMLBodyElement::new(HTMLElement::new(element_data))).into_type::<Element>()
        },
        "template" => DOMPtr::new(HTMLTemplateElement::new(HTMLElement::new(element_data)))
            .into_type::<Element>(),
        "script" => DOMPtr::new(HTMLScriptElement::new(HTMLElement::new(element_data)))
            .into_type::<Element>(),
        "noscript" => DOMPtr::new(HTMLNoscriptElement::new(HTMLElement::new(element_data)))
            .into_type::<Element>(),
        _ => {
            log::warn!("Failed to create element for interface {local_name:?}");
            DOMPtr::new(element_data)
        },
    }
}
