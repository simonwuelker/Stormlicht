use std::collections::HashMap;

use super::{
    BasicBlock, BasicBlockExit, Exception, Instruction, Program, Register, ThrowCompletionOr,
};
use crate::{
    value::{Object, StringOrNumericBinaryOperator},
    Value,
};

#[derive(Clone, Debug, Default)]
pub struct Vm {
    variables: HashMap<String, Value>,
    registers: Vec<Value>,
}

impl Vm {
    pub fn dump(&self) {
        println!("Registers:");
        for (index, reg) in self.registers.iter().enumerate() {
            println!("\t{index}. {reg:?}");
        }

        println!("Variables:");
        for (index, (key, value)) in self.variables.iter().enumerate() {
            println!("\t{index}. {key:?} -> {value:?}");
        }
    }

    pub fn execute_program(&mut self, program: &Program) {
        let mut basic_block_index = 0;
        loop {
            let block_to_execute = &program.basic_blocks[basic_block_index];
            self.execute_basic_block(block_to_execute);

            match block_to_execute.exit {
                BasicBlockExit::Terminate => break,
                BasicBlockExit::GoTo(index) => basic_block_index = index,
                BasicBlockExit::Branch {
                    branch_on,
                    if_true,
                    if_false,
                } => {
                    if self.register(branch_on).to_boolean() {
                        basic_block_index = if_true;
                    } else {
                        basic_block_index = if_false;
                    }
                },
            }
        }
    }

    fn execute_basic_block(&mut self, block: &BasicBlock) {
        self.registers
            .resize_with(block.registers_required, Default::default);

        for instruction in &block.instructions {
            if let Err(exception) = self.execute_instruction(instruction) {
                self.report_unhandled_exception(exception);
                break;
            }
        }
    }

    #[must_use]
    fn register(&self, register: Register) -> &Value {
        &self.registers[register.index()]
    }

    #[must_use]
    fn register_mut(&mut self, register: Register) -> &mut Value {
        &mut self.registers[register.index()]
    }

    fn set_register(&mut self, register: Register, value: Value) {
        self.registers[register.index()] = value;
    }

    fn set_variable(&mut self, name: &str, value: Value) {
        *self.variables.get_mut(name).expect("Variable not defined") = value;
    }

    fn report_unhandled_exception(&self, exception: Exception) {
        println!("Unhandled Exception: {:?}", exception.value());
    }

    fn execute_instruction(&mut self, instruction: &Instruction) -> ThrowCompletionOr<()> {
        match instruction {
            Instruction::LoadImmediate {
                destination,
                immediate,
            } => {
                self.set_register(*destination, immediate.clone());
            },
            Instruction::CreateVariable { name } => {
                self.variables.insert(name.clone(), Value::default());
            },
            Instruction::UpdateVariable { name, src } => {
                self.set_variable(name, self.register(*src).clone());
            },
            Instruction::LoadVariable { name, dst } => {
                let value = self
                    .variables
                    .get(name)
                    .expect("Variable not defined")
                    .clone();
                self.set_register(*dst, value);
            },
            Instruction::Add { lhs, rhs, dst } => {
                // https://262.ecma-international.org/14.0/#sec-addition-operator-plus-runtime-semantics-evaluation
                let result = Value::apply_string_or_numeric_binary_operator(
                    self.register(*lhs).clone(),
                    StringOrNumericBinaryOperator::Add,
                    self.register(*rhs).clone(),
                )?;

                self.set_register(*dst, result);
            },
            Instruction::Subtract { lhs, rhs, dst } => {
                // https://262.ecma-international.org/14.0/#sec-subtraction-operator-minus-runtime-semantics-evaluation
                let result = Value::apply_string_or_numeric_binary_operator(
                    self.register(*lhs).clone(),
                    StringOrNumericBinaryOperator::Subtract,
                    self.register(*rhs).clone(),
                )?;

                self.set_register(*dst, result);
            },
            Instruction::Multiply { lhs, rhs, dst } => {
                // https://262.ecma-international.org/14.0/#sec-multiplicative-operators-runtime-semantics-evaluation
                let result = Value::apply_string_or_numeric_binary_operator(
                    self.register(*lhs).clone(),
                    StringOrNumericBinaryOperator::Multiply,
                    self.register(*rhs).clone(),
                )?;

                self.set_register(*dst, result);
            },
            Instruction::Divide { lhs, rhs, dst } => {
                // https://262.ecma-international.org/14.0/#sec-multiplicative-operators-runtime-semantics-evaluation
                let result = Value::apply_string_or_numeric_binary_operator(
                    self.register(*lhs).clone(),
                    StringOrNumericBinaryOperator::Divide,
                    self.register(*rhs).clone(),
                )?;

                self.set_register(*dst, result);
            },
            Instruction::LooselyEqual { lhs, rhs, dst } => {
                let result = Value::is_loosely_equal(self.register(*lhs), self.register(*rhs))?;
                self.set_register(*dst, result.into());
            },
            Instruction::NotLooselyEqual { lhs, rhs, dst } => {
                let result = !Value::is_loosely_equal(self.register(*lhs), self.register(*rhs))?;
                self.set_register(*dst, result.into());
            },
            Instruction::Throw { value } => {
                let value = self.register(*value).clone();
                return Err(Exception::new(value));
            },
            Instruction::CreateDataPropertyOrThrow {
                object,
                property_key,
                property_value,
            } => {
                // https://262.ecma-international.org/14.0/#sec-createdatapropertyorthrow
                let property_value = self.register(*property_value).clone();

                let Value::Object(object) = self.register_mut(*object) else {
                    panic!("Cannot create property on non-object");
                };

                // 1. Let success be ? CreateDataProperty(O, P, V).
                let success = Object::create_data_property(object, property_key, property_value)?;

                // 2. If success is false, throw a TypeError exception.
                if !success {
                    // FIXME: This should be a TypeError
                    return Err(Exception::new(Value::Null));
                }
            },
            Instruction::ToNumber { src, dst } => {
                let value = self.register(*src);
                let result = value.to_number()?.into();
                self.set_register(*dst, result);
            },
            other => todo!("Implement instruction {other:?}"),
        }

        Ok(())
    }
}
