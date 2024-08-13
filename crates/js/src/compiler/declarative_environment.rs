use std::{cell::RefCell, collections::HashMap, rc::Rc};

/// Holds all the bindings within a scope
#[derive(Clone)]
pub struct DeclarativeEnvironment {
    outer: Option<Rc<Self>>,
    // FIXME: Can we restructure environments and not use a refcell here?
    bindings: RefCell<HashMap<String, usize>>,
}

impl Default for DeclarativeEnvironment {
    fn default() -> Self {
        Self {
            outer: None,
            bindings: RefCell::new(HashMap::default()),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Binding {
    /// How many environment record "jumps" have to be performend
    /// to get to the environment that defines this binding
    environment_index: usize,

    /// The index of the binding within the environment
    index: usize,
}

impl DeclarativeEnvironment {
    #[must_use]
    pub fn locate_binding(&self, identifier: &str) -> Option<Binding> {
        self.locate_binding_inner(identifier, 0)
    }

    #[must_use]
    fn locate_binding_inner(&self, identifier: &str, depth: usize) -> Option<Binding> {
        if let Some(index) = self.bindings.borrow().get(identifier) {
            let binding = Binding {
                environment_index: depth,
                index: *index,
            };

            Some(binding)
        } else if let Some(outer_environment) = &self.outer {
            outer_environment.locate_binding_inner(identifier, depth + 1)
        } else {
            None
        }
    }

    pub fn insert_binding(&self, identifier: &str) {
        let mut bindings = self.bindings.borrow_mut();
        let index = bindings.len();
        bindings.insert(identifier.to_string(), index);
    }
}
