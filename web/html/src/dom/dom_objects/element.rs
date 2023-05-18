//! <https://dom.spec.whatwg.org/#interface-element>

use dom_derive::inherit;

use crate::{dom::ElementCustomState, infra::Namespace};

use super::Node;

#[inherit(Node)]
pub struct Element {
    namespace: Namespace,
    namespace_prefix: Option<String>,
    local_name: String,
    custom_state: ElementCustomState,
    is: Option<String>,
}

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
}
