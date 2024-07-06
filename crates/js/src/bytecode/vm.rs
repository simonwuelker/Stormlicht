use std::collections::HashMap;

use super::{
    BasicBlock, BasicBlockExit, Exception, Instruction, Program, Register, ThrowCompletionOr,
};
use crate::{
    value::{
        evaluate_string_or_numeric_binary_expression, StringOrNumericBinaryOperator,
        ValueOrReference,
    },
    Value,
};

#[derive(Clone, Debug, Default)]
pub struct Vm {
    variables: HashMap<String, Value>,
    registers: Vec<ValueOrReference>,
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

    pub fn execute_program(&mut self, program: &Program) -> ThrowCompletionOr<()> {
        let mut basic_block_index = 0;
        loop {
            let block_to_execute = &program.basic_blocks[basic_block_index];
            self.execute_basic_block(block_to_execute)?;

            match block_to_execute.exit {
                BasicBlockExit::Terminate => break,
                BasicBlockExit::GoTo(index) => basic_block_index = index,
                BasicBlockExit::Branch {
                    branch_on,
                    if_true,
                    if_false,
                } => {
                    if self.register(branch_on).get_value()?.to_boolean() {
                        basic_block_index = if_true;
                    } else {
                        basic_block_index = if_false;
                    }
                },
            }
        }

        Ok(())
    }

    fn execute_basic_block(&mut self, block: &BasicBlock) -> ThrowCompletionOr<()> {
        self.registers
            .resize_with(block.registers_required, Default::default);

        for instruction in &block.instructions {
            self.execute_instruction(instruction)?;
        }

        Ok(())
    }

    #[must_use]
    fn register(&self, register: Register) -> &ValueOrReference {
        &self.registers[register.index()]
    }

    fn set_register(&mut self, register: Register, value: ValueOrReference) {
        self.registers[register.index()] = value;
    }

    fn set_variable(&mut self, name: &str, value: Value) {
        *self.variables.get_mut(name).expect("Variable not defined") = value;
    }

    fn execute_instruction(&mut self, instruction: &Instruction) -> ThrowCompletionOr<()> {
        match instruction {
            Instruction::LoadImmediate {
                destination,
                immediate,
            } => {
                self.set_register(*destination, immediate.clone().into());
            },
            Instruction::CreateVariable { name } => {
                self.variables.insert(name.clone(), Value::default());
            },
            Instruction::UpdateVariable { name, src } => {
                self.set_variable(name, self.register(*src).clone().get_value()?.into());
            },
            Instruction::LoadVariable { name, dst } => {
                let value = self
                    .variables
                    .get(name)
                    .expect("Variable not defined")
                    .clone();
                self.set_register(*dst, value.into());
            },
            Instruction::Add { lhs, rhs, dst } => {
                // https://262.ecma-international.org/14.0/#sec-addition-operator-plus-runtime-semantics-evaluation
                let result = evaluate_string_or_numeric_binary_expression(
                    self.register(*lhs).clone(),
                    StringOrNumericBinaryOperator::Add,
                    self.register(*rhs).clone(),
                )?;

                self.set_register(*dst, result.into());
            },
            Instruction::Subtract { lhs, rhs, dst } => {
                // https://262.ecma-international.org/14.0/#sec-subtraction-operator-minus-runtime-semantics-evaluation
                let result = evaluate_string_or_numeric_binary_expression(
                    self.register(*lhs).clone(),
                    StringOrNumericBinaryOperator::Subtract,
                    self.register(*rhs).clone(),
                )?;

                self.set_register(*dst, result.into());
            },
            Instruction::Multiply { lhs, rhs, dst } => {
                // https://262.ecma-international.org/14.0/#sec-multiplicative-operators-runtime-semantics-evaluation
                let result = evaluate_string_or_numeric_binary_expression(
                    self.register(*lhs).clone(),
                    StringOrNumericBinaryOperator::Multiply,
                    self.register(*rhs).clone(),
                )?;

                self.set_register(*dst, result.into());
            },
            Instruction::Divide { lhs, rhs, dst } => {
                // https://262.ecma-international.org/14.0/#sec-multiplicative-operators-runtime-semantics-evaluation
                let result = evaluate_string_or_numeric_binary_expression(
                    self.register(*lhs).clone(),
                    StringOrNumericBinaryOperator::Divide,
                    self.register(*rhs).clone(),
                )?;

                self.set_register(*dst, result.into());
            },
            Instruction::Modulo { lhs, rhs, dst } => {
                // https://262.ecma-international.org/14.0/#sec-multiplicative-operators-runtime-semantics-evaluation
                let result = evaluate_string_or_numeric_binary_expression(
                    self.register(*lhs).clone(),
                    StringOrNumericBinaryOperator::Modulo,
                    self.register(*rhs).clone(),
                )?;

                self.set_register(*dst, result.into());
            },
            Instruction::Exponentiate { lhs, rhs, dst } => {
                // https://262.ecma-international.org/14.0/#sec-exp-operator-runtime-semantics-evaluation
                let result = evaluate_string_or_numeric_binary_expression(
                    self.register(*lhs).clone(),
                    StringOrNumericBinaryOperator::Exponentiate,
                    self.register(*rhs).clone(),
                )?;

                self.set_register(*dst, result.into());
            },
            Instruction::BitwiseAnd { lhs, rhs, dst } => {
                // https://262.ecma-international.org/14.0/#sec-binary-bitwise-operators-runtime-semantics-evaluation
                let result = evaluate_string_or_numeric_binary_expression(
                    self.register(*lhs).clone(),
                    StringOrNumericBinaryOperator::BitwiseAnd,
                    self.register(*rhs).clone(),
                )?;

                self.set_register(*dst, result.into());
            },
            Instruction::BitwiseOr { lhs, rhs, dst } => {
                // https://262.ecma-international.org/14.0/#sec-binary-bitwise-operators-runtime-semantics-evaluation
                let result = evaluate_string_or_numeric_binary_expression(
                    self.register(*lhs).clone(),
                    StringOrNumericBinaryOperator::BitwiseOr,
                    self.register(*rhs).clone(),
                )?;

                self.set_register(*dst, result.into());
            },
            Instruction::BitwiseXor { lhs, rhs, dst } => {
                // https://262.ecma-international.org/14.0/#sec-binary-bitwise-operators-runtime-semantics-evaluation
                let result = evaluate_string_or_numeric_binary_expression(
                    self.register(*lhs).clone(),
                    StringOrNumericBinaryOperator::BitwiseExclusiveOr,
                    self.register(*rhs).clone(),
                )?;

                self.set_register(*dst, result.into());
            },
            Instruction::ShiftLeft { lhs, rhs, dst } => {
                // https://262.ecma-international.org/14.0/#sec-left-shift-operator-runtime-semantics-evaluation
                let result = evaluate_string_or_numeric_binary_expression(
                    self.register(*lhs).clone(),
                    StringOrNumericBinaryOperator::ShiftLeft,
                    self.register(*rhs).clone(),
                )?;

                self.set_register(*dst, result.into());
            },
            Instruction::ShiftRight { lhs, rhs, dst } => {
                // https://262.ecma-international.org/14.0/#sec-signed-right-shift-operator
                let result = evaluate_string_or_numeric_binary_expression(
                    self.register(*lhs).clone(),
                    StringOrNumericBinaryOperator::ShiftLeft,
                    self.register(*rhs).clone(),
                )?;

                self.set_register(*dst, result.into());
            },
            Instruction::ShiftRightZeros { lhs, rhs, dst } => {
                // https://262.ecma-international.org/14.0/#sec-unsigned-right-shift-operator
                let result = evaluate_string_or_numeric_binary_expression(
                    self.register(*lhs).clone(),
                    StringOrNumericBinaryOperator::ShiftLeft,
                    self.register(*rhs).clone(),
                )?;

                self.set_register(*dst, result.into());
            },
            Instruction::LogicalAnd { lhs, rhs, dst } => {
                // https://262.ecma-international.org/14.0/#sec-binary-logical-operators-runtime-semantics-evaluation
                let lval = self.register(*lhs).get_value()?;
                let result = if !lval.to_boolean() {
                    lval
                } else {
                    self.register(*rhs).get_value()?
                };

                self.set_register(*dst, result.into());
            },
            Instruction::LooselyEqual { lhs, rhs, dst } => {
                // https://262.ecma-international.org/14.0/#sec-equality-operators-runtime-semantics-evaluation
                let result = Value::is_loosely_equal(
                    &self.register(*lhs).get_value()?,
                    &self.register(*rhs).get_value()?,
                )?;
                self.set_register(*dst, Value::from(result).into());
            },
            Instruction::NotLooselyEqual { lhs, rhs, dst } => {
                // https://262.ecma-international.org/14.0/#sec-equality-operators-runtime-semantics-evaluation
                let result = !Value::is_loosely_equal(
                    &self.register(*lhs).get_value()?,
                    &self.register(*rhs).get_value()?,
                )?;
                self.set_register(*dst, Value::from(result).into());
            },
            Instruction::Throw { value } => {
                let value = self.register(*value).clone().get_value()?;
                return Err(Exception::new(value));
            },
            Instruction::MemberAccessWithIdentifier {
                base,
                identifier,
                dst,
            } => {
                // https://262.ecma-international.org/14.0/#sec-property-accessors-runtime-semantics-evaluation
                let base_value = self.register(*base).get_value()?;
                let result = Value::evaluate_property_access_with_identifier_key(
                    base_value,
                    identifier.clone(),
                );
                self.set_register(*dst, result.into());
            },
            other => todo!("Implement instruction {other:?}"),
        }

        Ok(())
    }
}
