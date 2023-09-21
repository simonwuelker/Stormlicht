//! <https://dom.spec.whatwg.org/>

mod boundary_point;
mod codegen;
mod display;
pub mod dom_objects;
mod dom_ptr;

pub use boundary_point::{BoundaryPoint, RelativePosition};
pub use codegen::{DOMType, DOMTyped};
pub use display::DOMDisplay;
pub use dom_ptr::{DOMPtr, WeakDOMPtr};
use string_interner::{static_interned, static_str, InternedString};

use crate::infra::Namespace;
use dom_objects::{
    Document, Element, HTMLAnchorElement, HTMLBodyElement, HTMLButtonElement, HTMLDivElement,
    HTMLElement, HTMLHeadElement, HTMLHtmlElement, HTMLLinkElement, HTMLMetaElement,
    HTMLNoscriptElement, HTMLParagraphElement, HTMLScriptElement, HTMLStyleElement,
    HTMLTemplateElement, HTMLTitleElement,
};

use self::dom_objects::{HTMLHeadingElement, HTMLLIElement};

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
    local_name: InternedString,
    namespace: Namespace,
    prefix: Option<InternedString>,
    is: Option<InternedString>,
    _synchronous_custom_elements_flag: bool,
) -> DOMPtr<Element> {
    // FIXME: make this spec-compliant!

    // 1. If prefix was not given, let prefix be null.
    // NOTE: we treat "not given" as null

    // 2. If is was not given, let is be null.
    // NOTE: we treat "not given" as null

    // 3. Let result be null.
    // NOTE: the part of this algorithm that is implemented does not require a result
    // let mut result = None;

    // 4. Let definition be the result of looking up a custom element definition given document, namespace, localName, and is.
    let definition = lookup_custom_element_definition(namespace, local_name, is);

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
                local_name,
                ElementCustomState::Uncustomized,
                None,
                is,
            );
            element_data.set_owning_document(document);

            let created_element = create_element_for_interface(local_name, namespace, element_data);

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
    _local_name: InternedString,
    _is: Option<InternedString>,
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
    local_name: InternedString,
    namespace: Namespace,
    element_data: Element,
) -> DOMPtr<Element> {
    if namespace != Namespace::HTML {
        todo!();
    }

    match local_name {
        static_interned!("a") => {
            DOMPtr::new(HTMLAnchorElement::new(HTMLElement::new(element_data)))
                .into_type::<Element>()
        },
        static_interned!("body") => {
            DOMPtr::new(HTMLBodyElement::new(HTMLElement::new(element_data))).into_type::<Element>()
        },
        static_interned!("button") => {
            DOMPtr::new(HTMLButtonElement::new(HTMLElement::new(element_data)))
                .into_type::<Element>()
        },
        static_interned!("div") => {
            DOMPtr::new(HTMLDivElement::new(HTMLElement::new(element_data))).into_type::<Element>()
        },
        static_interned!("head") => {
            DOMPtr::new(HTMLHeadElement::new(HTMLElement::new(element_data))).into_type::<Element>()
        },
        static_interned!("html") => {
            DOMPtr::new(HTMLHtmlElement::new(HTMLElement::new(element_data))).into_type::<Element>()
        },
        static_interned!("link") => {
            DOMPtr::new(HTMLLinkElement::new(HTMLElement::new(element_data))).into_type::<Element>()
        },
        static_interned!("meta") => {
            DOMPtr::new(HTMLMetaElement::new(HTMLElement::new(element_data))).into_type::<Element>()
        },
        static_interned!("noscript") => {
            DOMPtr::new(HTMLNoscriptElement::new(HTMLElement::new(element_data)))
                .into_type::<Element>()
        },
        static_interned!("p") => {
            DOMPtr::new(HTMLParagraphElement::new(HTMLElement::new(element_data)))
                .into_type::<Element>()
        },
        static_interned!("script") => {
            DOMPtr::new(HTMLScriptElement::new(HTMLElement::new(element_data)))
                .into_type::<Element>()
        },
        static_interned!("style") => {
            DOMPtr::new(HTMLStyleElement::new(HTMLElement::new(element_data)))
                .into_type::<Element>()
        },
        static_interned!("template") => {
            DOMPtr::new(HTMLTemplateElement::new(HTMLElement::new(element_data)))
                .into_type::<Element>()
        },
        static_interned!("title") => {
            DOMPtr::new(HTMLTitleElement::new(HTMLElement::new(element_data)))
                .into_type::<Element>()
        },
        static_interned!("h1")
        | static_interned!("h2")
        | static_interned!("h3")
        | static_interned!("h4")
        | static_interned!("h5")
        | static_interned!("h6") => {
            DOMPtr::new(HTMLHeadingElement::new(HTMLElement::new(element_data)))
                .into_type::<Element>()
        },
        static_interned!("li") => {
            DOMPtr::new(HTMLLIElement::new(HTMLElement::new(element_data))).into_type::<Element>()
        },
        _ => {
            log::warn!(
                "Failed to create element for interface {:?}",
                local_name.to_string()
            );
            DOMPtr::new(element_data)
        },
    }
}
