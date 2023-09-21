use crate::{
    dom::{dom_objects::Element, DOMPtr, DOMTyped},
    html::tokenization::TagData,
};

/// <https://html.spec.whatwg.org/multipage/parsing.html#the-list-of-active-formatting-elements>
#[derive(Clone, Default)]
pub struct ActiveFormattingElements {
    elements: Vec<FormatEntry>,
}

#[derive(Clone)]
pub enum FormatEntry {
    Marker,
    Element(ActiveFormattingElement),
}

#[derive(Clone)]
pub struct ActiveFormattingElement {
    pub element: DOMPtr<Element>,

    /// The tag that created this element
    pub tag: TagData,
}

impl FormatEntry {
    #[inline]
    #[must_use]
    pub fn is_marker(&self) -> bool {
        matches!(self, Self::Marker)
    }

    #[inline]
    #[must_use]
    pub fn as_element(&self) -> Option<ActiveFormattingElement> {
        match self {
            Self::Element(element) => Some(element.clone()),
            Self::Marker => None,
        }
    }
}

impl ActiveFormattingElements {
    /// <https://html.spec.whatwg.org/multipage/parsing.html#push-onto-the-list-of-active-formatting-elements>
    pub fn push(&mut self, element: DOMPtr<Element>, tag: TagData) {
        // 1. If there are already three elements in the list of active formatting elements after the last marker,
        //    if any, or anywhere in the list if there are no markers, that have the same tag name, namespace, and attributes as element,
        //    then remove the earliest such element from the list of active formatting elements.
        //    For these purposes, the attributes must be compared as they were when the elements were created by the parser;
        //    two elements have the same attributes if all their parsed attributes can be paired such that the two attributes
        //    in each pair have identical names, namespaces, and values (the order of the attributes does not matter).

        // Contains Some((index_of_first_match, nr_of_matches)) if at least one match was found
        let mut elements_found = None;

        for (index, active_formatting_element) in self.elements[self.last_marker()..]
            .iter()
            .enumerate()
            .filter_map(|(i, element)| Some((i, element.as_element()?)))
        {
            let is_equal_element = {
                let active_formatting_element = active_formatting_element.element.borrow();
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
                        self.elements
                            .remove(self.last_marker() + *index_of_first_match);
                        break;
                    },
                    None => {
                        elements_found = Some((index, 1));
                    },
                }
            }
        }

        // 2. Add element to the list of active formatting elements.
        self.elements
            .push(FormatEntry::Element(ActiveFormattingElement {
                element,
                tag,
            }));
    }

    #[inline]
    #[must_use]
    pub fn last(&self) -> Option<&FormatEntry> {
        self.elements.last()
    }

    #[inline]
    #[must_use]
    pub fn elements(&self) -> &[FormatEntry] {
        &self.elements
    }

    #[inline]
    #[must_use]
    pub fn elements_mut(&mut self) -> &mut [FormatEntry] {
        &mut self.elements
    }

    #[inline]
    pub fn list(&self) -> impl Iterator<Item = ActiveFormattingElement> + '_ {
        self.elements.iter().filter_map(FormatEntry::as_element)
    }

    /// Return the last marker or `0` if there are no markers
    #[inline]
    #[must_use]
    fn last_marker(&self) -> usize {
        self.elements
            .iter()
            .rposition(FormatEntry::is_marker)
            .unwrap_or(0)
    }

    pub fn remove_element_at_index_from_last_marker(&mut self, i: usize) {
        self.elements.remove(self.last_marker() + i);
    }

    /// Return all elements between the end of the list and the last marker on the list (or the start of the list if there is no marker on the list)
    #[inline]
    pub fn elements_since_last_marker(&self) -> impl Iterator<Item = ActiveFormattingElement> + '_ {
        self.elements[self.last_marker()..]
            .iter()
            .filter_map(FormatEntry::as_element)
    }

    #[inline]
    pub fn remove<T: DOMTyped>(&mut self, to_remove: &DOMPtr<T>) {
        self.elements
            .retain(|element_or_marker| match element_or_marker {
                FormatEntry::Element(formatting_element) => {
                    !DOMPtr::ptr_eq(to_remove, &formatting_element.element)
                },
                FormatEntry::Marker => true,
            })
    }

    #[must_use]
    pub fn find<T: DOMTyped>(&self, needle: &DOMPtr<T>) -> Option<usize> {
        self.elements
            .iter()
            .enumerate()
            .filter_map(|(i, entry)| Some((i, entry.as_element()?)))
            .find(|(_, element)| DOMPtr::ptr_eq(&element.element, needle))
            .map(|(i, _)| i)
    }

    #[inline]
    pub fn contains<T: DOMTyped>(&self, needle: &DOMPtr<T>) -> bool {
        self.find(needle).is_some()
    }

    #[inline]
    pub fn push_marker(&mut self) {
        self.elements.push(FormatEntry::Marker)
    }
}
