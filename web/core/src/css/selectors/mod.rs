//! <https://drafts.csswg.org/selectors-4/>

mod any_value;
mod attribute_matcher;
mod attribute_modifier;
mod attribute_selector;
mod class_selector;
mod combinator;
mod complex_real_selector;
mod complex_selector;
mod complex_selector_unit;
mod compound_selector;
mod id_selector;
mod legacy_pseudo_element_selector;
mod ns_prefix;
mod pseudo_class_selector;
mod pseudo_compound_selector;
mod pseudo_element_selector;
mod relative_real_selector;
mod relative_selector;
mod simple_selector;
mod specificity;
mod subclass_selector;
mod type_selector;
mod wq_name;

pub use any_value::AnyValue;
pub use attribute_matcher::AttributeMatcher;
pub use attribute_modifier::AttributeModifier;
pub use attribute_selector::AttributeSelector;
pub use class_selector::ClassSelector;
pub use combinator::Combinator;
pub use complex_real_selector::{ComplexRealSelector, ComplexRealSelectorList};
pub use complex_selector::{ComplexSelector, ComplexSelectorList, SelectorList};
pub use complex_selector_unit::ComplexSelectorUnit;
pub use compound_selector::{CompoundSelector, CompoundSelectorList};
pub use id_selector::IDSelector;
pub use legacy_pseudo_element_selector::LegacyPseudoElementSelector;
pub use ns_prefix::NSPrefix;
pub use pseudo_class_selector::PseudoClassSelector;
pub use pseudo_compound_selector::PseudoCompoundSelector;
pub use pseudo_element_selector::PseudoElementSelector;
pub use relative_real_selector::{RelativeRealSelector, RelativeRealSelectorList};
pub use relative_selector::{RelativeSelector, RelativeSelectorList};
pub use simple_selector::{SimpleSelector, SimpleSelectorList};
pub use specificity::Specificity;
pub use subclass_selector::SubClassSelector;
pub use type_selector::TypeSelector;
pub use wq_name::WQName;

use std::fmt;

use crate::dom::{dom_objects::Element, DOMPtr};

use super::Serializer;

pub fn serialize_selector_list<S: Serializer>(
    selectors: &[ComplexSelector],
    mut serializer: S,
) -> fmt::Result {
    serializer.serialize_comma_seperated_list(selectors)
}

pub trait CSSValidateSelector {
    /// <https://drafts.csswg.org/selectors-4/#invalid-selector>
    fn is_valid(&self) -> bool;
}

pub trait Selector {
    /// Determine if the given selector matches the given element
    fn matches(&self, element: &DOMPtr<Element>) -> bool;

    /// Calculate the selectors [Specificity](https://drafts.csswg.org/selectors-4/#specificity)
    fn specificity(&self) -> Specificity;
}
