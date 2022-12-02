use crate::js_type::JSType;
use std::collections::HashMap;

/// Traverses and executes a js ast
pub struct JSInterpreter {
    variable_store: HashMap<String, JSType>,
}

impl Default for JSInterpreter {
    fn default() -> Self {
        Self {
            variable_store: HashMap::new(),
        }
    }
}
