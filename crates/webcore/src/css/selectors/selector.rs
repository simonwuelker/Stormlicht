use std::fmt;

use crate::{
    css::{
        selectors::{ComplexSelector, Specificity},
        Serializer,
    },
    dom::{dom_objects::Element, DomPtr},
};

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
    fn matches(&self, element: &DomPtr<Element>) -> bool;

    /// Calculate the selectors [Specificity](https://drafts.csswg.org/selectors-4/#specificity)
    fn specificity(&self) -> Specificity;
}
