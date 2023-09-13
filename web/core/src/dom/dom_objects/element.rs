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

    intrinsic_size: Option<math::Rectangle>,
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

    #[inline]
    #[must_use]
    pub fn local_name(&self) -> InternedString {
        self.local_name
    }

    #[inline]
    #[must_use]
    pub fn namespace(&self) -> Namespace {
        self.namespace
    }

    pub fn id(&self) -> InternedString {
        self.id
    }

    pub fn is_replaced(&self) -> bool {
        self.intrinsic_size.is_some()
    }

    pub fn intrinsic_size(&self) -> Option<math::Rectangle> {
        self.intrinsic_size
    }
}
