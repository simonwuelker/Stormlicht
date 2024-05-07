//! <https://drafts.csswg.org/selectors-4/>

// Not all of these selectors are in use right now, but they are all
// in the specification so presumably we will need them at some point
#![allow(unused_imports)]

mod attribute_matcher;
mod attribute_modifier;
mod attribute_selector;
mod combinator;
mod namespace_prefix;
mod pseudo_class_selector;
mod qualified_name;
mod specificity;
mod type_selector;

pub use attribute_matcher::AttributeMatcher;
pub use attribute_modifier::AttributeModifier;
pub use attribute_selector::AttributeSelector;
pub use combinator::Combinator;
pub use namespace_prefix::NamespacePrefix;
pub use pseudo_class_selector::PseudoClassSelector;
pub use qualified_name::WellQualifiedName;
pub use specificity::Specificity;
pub use type_selector::TypeSelector;

mod selector;

pub use selector::{CSSValidateSelector, Selector};
