use crate::dom::{dom_objects::Element, DOMPtr, DOMTyped};

/// <https://html.spec.whatwg.org/multipage/parsing.html#the-list-of-active-formatting-elements>
#[derive(Clone, Default)]
pub struct ActiveFormattingElements {
    elements: Vec<DOMPtr<Element>>,
    markers: Vec<usize>,
}

impl ActiveFormattingElements {
    /// <https://html.spec.whatwg.org/multipage/parsing.html#push-onto-the-list-of-active-formatting-elements>
    pub fn push(&mut self, element: DOMPtr<Element>) {
        // 1. If there are already three elements in the list of active formatting elements after the last marker,
        //    if any, or anywhere in the list if there are no markers, that have the same tag name, namespace, and attributes as element,
        //    then remove the earliest such element from the list of active formatting elements.
        //    For these purposes, the attributes must be compared as they were when the elements were created by the parser;
        //    two elements have the same attributes if all their parsed attributes can be paired such that the two attributes
        //    in each pair have identical names, namespaces, and values (the order of the attributes does not matter).

        // Contains Some((index_of_first_match, nr_of_matches)) if at least one match was found
        let mut elements_found = None;

        for (index, active_formatting_element) in
            self.elements_since_last_marker().iter().enumerate()
        {
            let is_equal_element = {
                let active_formatting_element = active_formatting_element.borrow();
                let element = element.borrow();

                // FIXME: Compare the attributes too!
                element.local_name() == active_formatting_element.local_name()
                    && element.namespace() == active_formatting_element.namespace()
            };

            if is_equal_element {
                match &mut elements_found {
                    Some((_, n_matches @ ..3)) => {
                        *n_matches += 1;
                    },
                    Some((index_of_first_match, 3..)) => {
                        // FIXME: do we need to update marker positions after this?
                        self.elements.remove(*index_of_first_match);
                        break;
                    },
                    None => {
                        elements_found = Some((index, 1));
                    },
                }
            }
        }

        // 2. Add element to the list of active formatting elements.
        self.elements.push(element);
    }

    #[must_use]
    #[inline]
    pub fn list(&self) -> &[DOMPtr<Element>] {
        &self.elements
    }

    /// Return the last marker or `0` if there are no markers
    #[inline]
    #[must_use]
    fn last_marker(&self) -> usize {
        self.markers.last().copied().unwrap_or(0)
    }

    pub fn remove_element_at_index_from_last_marker(&mut self, i: usize) {
        self.elements.remove(self.last_marker() + i);
    }

    /// Return all elements between the end of the list and the last marker on the list (or the start of the list if there is no marker on the list)
    #[inline]
    #[must_use]
    pub fn elements_since_last_marker(&self) -> &[DOMPtr<Element>] {
        &self.elements[self.last_marker()..]
    }

    #[inline]
    pub fn remove<T: DOMTyped>(&mut self, to_remove: &DOMPtr<T>) {
        // FIXME: do we need to update marker positions after this?
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

    #[inline]
    pub fn contains<T: DOMTyped>(&self, needle: &DOMPtr<T>) -> bool {
        self.find(needle).is_some()
    }

    #[inline]
    pub fn insert_marker(&mut self) {
        self.markers.push(self.elements.len())
    }
}
