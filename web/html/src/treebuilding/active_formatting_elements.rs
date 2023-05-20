use crate::dom::{dom_objects::Element, DOMPtr, DOMTyped};

/// <https://html.spec.whatwg.org/multipage/parsing.html#the-list-of-active-formatting-elements>
#[derive(Clone, Default)]
pub struct ActiveFormattingElements {
    elements: Vec<DOMPtr<Element>>,
    markers: Vec<usize>,
}

impl ActiveFormattingElements {
    /// <https://html.spec.whatwg.org/multipage/parsing.html#push-onto-the-list-of-active-formatting-elements>
    pub fn push(&mut self, _element: DOMPtr<Element>) {
        log::warn!("FIXME: Implement push-onto-the-list-of-active-formatting-elements algorithm")
    }

    #[must_use]
    #[inline]
    pub fn list(&self) -> &[DOMPtr<Element>] {
        &self.elements
    }

    /// Return the last marker or `0` if there are no markers
    #[must_use]
    fn last_marker(&self) -> usize {
        self.markers.last().copied().unwrap_or(0)
    }

    pub fn remove_element_at_index_from_last_marker(&mut self, i: usize) {
        self.elements.remove(self.last_marker() + i);
    }

    /// Return all elements between the end of the list and the last marker on the list (or the start of the list if there is no marker on the list)
    #[must_use]
    #[inline]
    pub fn elements_since_last_marker(&self) -> &[DOMPtr<Element>] {
        &self.elements[self.last_marker()..]
    }

    pub fn remove<T: DOMTyped>(&mut self, to_remove: &DOMPtr<T>) {
        self.elements
            .retain_mut(|element| !DOMPtr::ptr_eq(to_remove, element))
    }

    #[must_use]
    pub fn find<T: DOMTyped>(&self, needle: &DOMPtr<T>) -> Option<usize> {
        self.elements
            .iter()
            .enumerate()
            .find(|(_, node)| node.ptr_eq(needle))
            .map(|(i, _)| i)
    }

    pub fn contains<T: DOMTyped>(&self, needle: &DOMPtr<T>) -> bool {
        self.find(needle).is_some()
    }
}
