use crate::{compiler, Value};

#[derive(Clone, Debug, Default)]
pub struct LexicalEnvironment {
    outer: Option<Box<Self>>,
    variables: Vec<Value>,
}

impl LexicalEnvironment {
    pub fn reserve_variables(&mut self, num_variables: usize) {
        self.variables.resize(num_variables, Value::Undefined);
    }

    /// Resolve a binding
    ///
    /// # Panics
    /// This function panics if the binding references a non-existent variable
    #[must_use]
    pub fn get_binding_mut(&mut self, binding: compiler::Binding) -> &mut Value {
        let mut environment = self;

        for _ in 0..binding.environment_index {
            environment = environment
                .outer
                .as_mut()
                .expect("environment does not exist");
        }

        &mut environment.variables[binding.index]
    }
}
