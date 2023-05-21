use dom_derive::inherit;

use crate::{display_string, dom::ElementCustomState, infra::Namespace};

use super::Node;

/// <https://dom.spec.whatwg.org/#interface-element>
#[inherit(Node)]
pub struct Element {
    namespace: Namespace,
    namespace_prefix: Option<String>,
    local_name: String,
    custom_state: ElementCustomState,
    is: Option<String>,
}

display_string!(Element, "ELEMENT");

impl Element {
    pub fn new(
        namespace: Namespace,
        namespace_prefix: Option<String>,
        local_name: String,
        custom_state: ElementCustomState,
        _custom_element_definition: Option<()>,
        is: Option<String>,
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

    pub fn local_name(&self) -> &str {
        &self.local_name
    }
}
