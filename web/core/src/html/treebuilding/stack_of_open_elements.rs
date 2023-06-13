use crate::dom::{dom_objects::Node, DOMPtr, DOMType, DOMTyped};

#[derive(Clone, Default)]
/// <https://html.spec.whatwg.org/multipage/parsing.html#stack-of-open-elements>
///
/// This is a wrapper struct around `Vec<DOMPtr<Node>>` because the terminology in the html
/// spec does not match the one used for `Vec<T>` (a html stack grows "downwards") which
/// can lead to subtle bugs.
pub struct StackOfOpenElements {
    open_elements: Vec<DOMPtr<Node>>,
}

impl StackOfOpenElements {
    pub fn push(&mut self, node: DOMPtr<Node>) {
        self.open_elements.push(node);
    }

    #[must_use]
    pub fn top_node(&self) -> Option<DOMPtr<Node>> {
        self.open_elements.first().cloned()
    }

    #[must_use]
    pub fn bottommost_node(&self) -> Option<DOMPtr<Node>> {
        self.open_elements.last().cloned()
    }

    pub fn pop(&mut self) -> Option<DOMPtr<Node>> {
        let popped_element_or_none = self.open_elements.pop();
        if let Some(popped_element) = &popped_element_or_none {
            if popped_element.underlying_type() == DOMType::HTMLStyleElement {
                log::info!("popping style element");
            }
        }
        popped_element_or_none
    }

    #[must_use]
    pub fn find<T: DOMTyped>(&self, needle: &DOMPtr<T>) -> Option<usize> {
        self.open_elements
            .iter()
            .enumerate()
            .find(|(_, node)| node.ptr_eq(needle))
            .map(|(i, _)| i)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.open_elements.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.open_elements.is_empty()
    }

    #[must_use]
    pub fn contains<T: DOMTyped>(&self, needle: &DOMPtr<T>) -> bool {
        self.find(needle).is_some()
    }

    #[must_use]
    pub fn list(&self) -> &[DOMPtr<Node>] {
        &self.open_elements
    }

    pub fn remove<T: DOMTyped>(&mut self, to_remove: &DOMPtr<T>) {
        self.open_elements
            .retain_mut(|element| !DOMPtr::ptr_eq(to_remove, element))
    }
}
