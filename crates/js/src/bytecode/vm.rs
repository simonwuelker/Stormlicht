use std::collections::HashMap;

use super::{
    BasicBlock, BasicBlockExit, Exception, Instruction, Program, Register, ThrowCompletionOr,
};
use crate::Value;

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

    fn set_register(&mut self, register: Register, value: Value) {
        self.registers[register.index()] = value;
    }

    fn set_variable(&mut self, name: &str, value: Value) {
        *self.variables.get_mut(name).expect("Variable not defined") = value;
    }

    fn report_unhandled_exception(&self, exception: Exception) {
        println!("Unhandled Exception: {exception:?}");
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
                // <https://262.ecma-international.org/14.0/#sec-applystringornumericbinaryoperator>
                let lprim = self.register(*lhs).to_primitive(None)?;
                let rprim = self.register(*rhs).to_primitive(None)?;

                if lprim.is_string() || rprim.is_string() {
                    // i. Let lstr be ? ToString(lprim).
                    let lstr = lprim.to_string()?;

                    // ii. Let rstr be ? ToString(rprim).
                    let rstr = rprim.to_string()?;

                    // iii. Return the string-concatenation of lstr and rstr.
                    self.set_register(*dst, format!("{lstr}{rstr}").into());
                    return Ok(());
                }

                let lval = lprim;
                let rval = rprim;

                // 3. Let lnum be ? ToNumeric(lval).
                let lnum = lval.to_numeric()?;

                // 4. Let rnum be ? ToNumeric(rval).
                let rnum = rval.to_numeric()?;

                // 5. If Type(lnum) is not Type(rnum), throw a TypeError exception.
                if lnum.type_tag() != rnum.type_tag() {
                    return Err(Exception::TypeError);
                }

                match (lnum, rnum) {
                    (Value::Number(lhs), Value::Number(rhs)) => {
                        self.set_register(*dst, lhs.add(rhs).into());
                    },
                    (Value::BigInt, Value::BigInt) => todo!(),
                    _ => unreachable!(),
                }
            },
            _ => todo!(),
        }

        Ok(())
    }
}
