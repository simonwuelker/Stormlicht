use std::collections::HashMap;

use dom_derive::inherit;
use string_interner::{static_interned, static_str, InternedString};

use crate::{dom::ElementCustomState, infra::Namespace};

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
    attributes: HashMap<InternedString, InternedString>,

    intrinsic_size: Option<math::Rectangle>,
}

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

    /// <https://dom.spec.whatwg.org/#concept-element-attributes-append>
    #[inline]
    pub fn append_attribute(&mut self, key: InternedString, value: InternedString) {
        self.attributes.insert(key, value);
    }

    #[inline]
    pub fn attributes(&self) -> &HashMap<InternedString, InternedString> {
        &self.attributes
    }

    #[inline]
    pub fn attributes_mut(&mut self) -> &mut HashMap<InternedString, InternedString> {
        &mut self.attributes
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

    #[inline]
    pub fn id(&self) -> Option<InternedString> {
        self.attributes.get(&static_interned!("id")).copied()
    }

    pub fn is_replaced(&self) -> bool {
        self.intrinsic_size.is_some()
    }

    pub fn intrinsic_size(&self) -> Option<math::Rectangle> {
        self.intrinsic_size
    }
}
