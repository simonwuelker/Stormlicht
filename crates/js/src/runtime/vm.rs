use crate::{compiler, value::StringOrNumericBinaryOperator, Value};

use super::{Executable, LexicalEnvironment, OpCode};

#[derive(Clone, Debug, Default)]
pub struct Vm {
    program_counter: usize,
    stack: Vec<Value>,
    lexical_environment: LexicalEnvironment,
}

impl Vm {
    pub fn execute(&mut self, executable: Executable) {
        self.lexical_environment
            .reserve_variables(executable.num_variables);

        while let Some(instruction) = executable.fetch_instruction(self.program_counter) {
            self.program_counter += 1;

            match instruction {
                OpCode::Add => self.add(),
                OpCode::Subtract => self.subtract(),
                OpCode::Jump(address) => self.jump(address),
                OpCode::LoadConstant(handle) => {
                    self.push(executable.fetch_constant(handle).clone())
                },
                OpCode::LoadFrom(binding) => self.load_from(binding),
                OpCode::StoreTo(binding) => self.store_to(binding),
                _ => todo!(),
            }
        }
    }

    /// Pop the top element off the stack
    ///
    /// # Panics
    /// This function panics if the stack is empty
    fn pop(&mut self) -> Value {
        self.stack.pop().expect("Stack cannot be empty")
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value)
    }

    /// Execute [OpCode::Add]
    fn add(&mut self) {
        let a = self.pop();
        let b = self.pop();
        let value = Value::apply_string_or_numeric_binary_operator(
            a,
            StringOrNumericBinaryOperator::Add,
            b,
        )
        .unwrap(); // FIXME
        self.push(value);
    }

    /// Execute [OpCode::Subtract]
    fn subtract(&mut self) {
        let a = self.pop();
        let b = self.pop();
        let value = Value::apply_string_or_numeric_binary_operator(
            a,
            StringOrNumericBinaryOperator::Subtract,
            b,
        )
        .unwrap(); // FIXME
        self.push(value);
    }

    /// Execute [OpCode::Jump]
    fn jump(&mut self, address: usize) {
        self.program_counter = address;
    }

    /// Execute [OpCode::LoadFrom]
    fn load_from(&mut self, binding: compiler::Binding) {
        let value = self.lexical_environment.get_binding_mut(binding).to_owned();
        self.push(value);
    }

    /// Execute [OpCode::StoreTo]
    fn store_to(&mut self, binding: compiler::Binding) {
        let value = self.pop();
        let variable = self.lexical_environment.get_binding_mut(binding);
        *variable = value;
    }
}
