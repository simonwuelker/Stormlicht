use std::{collections::HashMap, fmt};

use dom_derive::inherit;

use crate::{dom::ElementCustomState, infra::Namespace, static_interned, InternedString};

use super::Node;

/// Bitflag for states like active, hovered
///
/// Used for the CSS `:hover`/`:active` (and other) pseudoclasses
#[derive(Clone, Copy, Default)]
struct ElementFlags(u8);

impl ElementFlags {
    // If you add fields here, update the Debug impl below
    const HOVER: u8 = 1;

    #[inline]
    fn set(&mut self, flag: u8) {
        self.0 |= flag;
    }

    #[inline]
    fn unset(&mut self, flag: u8) {
        self.0 &= !flag;
    }

    #[inline]
    #[must_use]
    const fn is_set(&self, flag: u8) -> bool {
        self.0 & flag != 0
    }
}

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
    flags: ElementFlags,

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

    #[inline]
    pub fn set_hovered(&mut self, hovered: bool) {
        if hovered {
            self.flags.set(ElementFlags::HOVER)
        } else {
            self.flags.unset(ElementFlags::HOVER)
        }
    }

    #[inline]
    #[must_use]
    pub fn is_hovered(&self) -> bool {
        self.flags.is_set(ElementFlags::HOVER)
    }
}

impl fmt::Debug for ElementFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_set(Self::HOVER) {
            "HOVER".fmt(f)
        } else {
            "(empty)".fmt(f)
        }
    }
}
