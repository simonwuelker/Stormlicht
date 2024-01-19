use std::{collections::HashMap, mem};

use super::{vm, Instruction, Value, VariableHandle};

#[derive(Clone, Copy, Debug)]
pub struct Register(usize);

impl Register {
    #[must_use]
    pub const fn index(&self) -> usize {
        self.0
    }
}

#[derive(Clone, Debug, Default)]
pub struct Builder {
    variables: HashMap<String, VariableHandle>,
    registers_used: usize,
    variables_used: usize,
    instructions: Vec<Instruction>,
}

impl Builder {
    #[must_use]
    pub fn allocate_register(&mut self) -> Register {
        let register = Register(self.registers_used);
        self.registers_used += 1;

        register
    }

    #[must_use]
    pub fn allocate_register_with_value(&mut self, value: Value) -> Register {
        let register = self.allocate_register();
        let instruction = Instruction::LoadImmediate {
            destination: register,
            immediate: value,
        };
        self.instructions.push(instruction);
        register
    }

    #[must_use]
    pub fn finish_basic_block(&mut self) -> vm::BasicBlock {
        let instructions = mem::take(&mut self.instructions);
        let registers_required = self.registers_used;
        self.registers_used = 0;

        vm::BasicBlock {
            registers_required,
            instructions,
        }
    }

    #[must_use]
    pub fn create_variable(&mut self, name: &str) -> VariableHandle {
        let handle = VariableHandle::new(self.variables_used);
        self.variables.insert(name.to_string(), handle);
        self.variables_used += 1;

        let instruction = Instruction::CreateVariable { handle };
        self.instructions.push(instruction);

        handle
    }

    pub fn update_variable(&mut self, handle: VariableHandle, src: Register) {
        let instruction = Instruction::UpdateVariable { handle, src };
        self.instructions.push(instruction);
    }

    #[must_use]
    pub fn add(&mut self, lhs: Register, rhs: Register) -> Register {
        let dst = self.allocate_register();
        let instruction = Instruction::Add { lhs, rhs, dst };
        self.instructions.push(instruction);
        dst
    }

    #[must_use]
    pub fn subtract(&mut self, lhs: Register, rhs: Register) -> Register {
        let dst = self.allocate_register();
        let instruction = Instruction::Subtract { lhs, rhs, dst };
        self.instructions.push(instruction);
        dst
    }

    #[must_use]
    pub fn multiply(&mut self, lhs: Register, rhs: Register) -> Register {
        let dst = self.allocate_register();
        let instruction = Instruction::Multiply { lhs, rhs, dst };
        self.instructions.push(instruction);
        dst
    }

    #[must_use]
    pub fn divide(&mut self, lhs: Register, rhs: Register) -> Register {
        let dst = self.allocate_register();
        let instruction = Instruction::Divide { lhs, rhs, dst };
        self.instructions.push(instruction);
        dst
    }

    #[must_use]
    pub fn bitwise_or(&mut self, lhs: Register, rhs: Register) -> Register {
        let dst = self.allocate_register();
        let instruction = Instruction::BitwiseOr { lhs, rhs, dst };
        self.instructions.push(instruction);
        dst
    }

    #[must_use]
    pub fn bitwise_and(&mut self, lhs: Register, rhs: Register) -> Register {
        let dst = self.allocate_register();
        let instruction = Instruction::BitwiseAnd { lhs, rhs, dst };
        self.instructions.push(instruction);
        dst
    }

    #[must_use]
    pub fn bitwise_xor(&mut self, lhs: Register, rhs: Register) -> Register {
        let dst = self.allocate_register();
        let instruction = Instruction::BitwiseXor { lhs, rhs, dst };
        self.instructions.push(instruction);
        dst
    }

    #[must_use]
    pub fn logical_and(&mut self, lhs: Register, rhs: Register) -> Register {
        let dst = self.allocate_register();
        let instruction = Instruction::LogicalAnd { lhs, rhs, dst };
        self.instructions.push(instruction);
        dst
    }

    #[must_use]
    pub fn logical_or(&mut self, lhs: Register, rhs: Register) -> Register {
        let dst = self.allocate_register();
        let instruction = Instruction::LogicalOr { lhs, rhs, dst };
        self.instructions.push(instruction);
        dst
    }
}
