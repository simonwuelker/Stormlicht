use dom_derive::inherit;
use string_interner::InternedString;

use crate::{display_string, dom::ElementCustomState, infra::Namespace};

use super::Node;

/// <https://dom.spec.whatwg.org/#interface-element>
#[inherit(Node)]
pub struct Element {
    namespace: Namespace,
    namespace_prefix: Option<InternedString>,
    local_name: InternedString,
    custom_state: ElementCustomState,
    is: Option<InternedString>,
    id: InternedString,
}

display_string!(Element, "ELEMENT");

impl Element {
    pub fn new(
        namespace: Namespace,
        namespace_prefix: Option<InternedString>,
        local_name: InternedString,
        custom_state: ElementCustomState,
        _custom_element_definition: Option<()>,
        is: Option<InternedString>,
    ) -> Self {
        Self {
            namespace,
            namespace_prefix,
            local_name,
            custom_state,
            is,
            ..Default::default()
        }
    }

    pub fn local_name(&self) -> InternedString {
        self.local_name
    }

    pub fn id(&self) -> InternedString {
        self.id
    }
}
