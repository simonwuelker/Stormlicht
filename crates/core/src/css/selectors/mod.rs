//! <https://drafts.csswg.org/selectors-4/>

// Not all of these selectors are in use right now, but they are all
// in the specification so presumably we will need them at some point
#![allow(unused_imports)]

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
mod namespace_prefix;
mod pseudo_class_selector;
mod pseudo_compound_selector;
mod pseudo_element_selector;
mod qualified_name;
mod relative_real_selector;
mod relative_selector;
mod simple_selector;
mod specificity;
mod subclass_selector;
mod type_selector;

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
pub use namespace_prefix::NamespacePrefix;
pub use pseudo_class_selector::PseudoClassSelector;
pub use pseudo_compound_selector::PseudoCompoundSelector;
pub use pseudo_element_selector::PseudoElementSelector;
pub use qualified_name::WellQualifiedName;
pub use relative_real_selector::{RelativeRealSelector, RelativeRealSelectorList};
pub use relative_selector::{RelativeSelector, RelativeSelectorList};
pub use simple_selector::{SimpleSelector, SimpleSelectorList};
pub use specificity::Specificity;
pub use subclass_selector::SubClassSelector;
pub use type_selector::TypeSelector;

mod selector;

pub use selector::{serialize_selector_list, CSSValidateSelector, Selector};
