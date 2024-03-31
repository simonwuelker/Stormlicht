use super::{BasicBlock, BasicBlockExit, Instruction, Program};
use crate::{value::object, Value};

#[derive(Clone, Copy, Debug)]
pub struct Register(usize);

impl Register {
    #[must_use]
    pub const fn index(&self) -> usize {
        self.0
    }
}

#[derive(Clone, Debug)]
pub struct ProgramBuilder {
    current_block: usize,
    basic_blocks: Vec<BasicBlock>,
}

impl Default for ProgramBuilder {
    fn default() -> Self {
        Self {
            current_block: 0,
            basic_blocks: vec![BasicBlock::default()],
        }
    }
}

impl ProgramBuilder {
    #[must_use]
    pub fn get_current_block<'a>(&'a mut self) -> BasicBlockBuilder<'a> {
        let index = self.current_block();
        self.get_block(index)
    }

    #[must_use]
    pub fn current_block(&self) -> usize {
        self.current_block
    }

    pub fn set_current_block(&mut self, current_block: usize) {
        self.current_block = current_block;
    }

    #[must_use]
    pub fn get_block<'a>(&'a mut self, index: usize) -> BasicBlockBuilder<'a> {
        BasicBlockBuilder::new(&mut self.basic_blocks[index])
    }

    #[must_use]
    pub fn allocate_basic_block(&mut self) -> usize {
        let index = self.basic_blocks.len();
        self.basic_blocks.push(BasicBlock::default());
        index
    }

    #[must_use]
    pub fn finish(self) -> Program {
        Program {
            basic_blocks: self.basic_blocks,
        }
    }
}

#[derive(Debug)]
pub struct BasicBlockBuilder<'a> {
    basic_block: &'a mut BasicBlock,
}

impl<'a> BasicBlockBuilder<'a> {
    pub fn new(basic_block: &'a mut BasicBlock) -> Self {
        Self { basic_block }
    }

    #[must_use]
    pub fn allocate_register(&mut self) -> Register {
        let register = Register(self.basic_block.registers_required);
        self.basic_block.registers_required += 1;

        register
    }

    fn push_instruction(&mut self, instruction: Instruction) {
        self.basic_block.instructions.push(instruction);
    }

    #[must_use]
    pub fn allocate_register_with_value(&mut self, value: Value) -> Register {
        let register = self.allocate_register();
        let instruction = Instruction::LoadImmediate {
            destination: register,
            immediate: value,
        };
        self.push_instruction(instruction);
        register
    }

    pub fn create_variable(&mut self, name: &str) {
        let instruction = Instruction::CreateVariable {
            name: name.to_string(),
        };
        self.push_instruction(instruction);
    }

    pub fn update_variable(&mut self, name: String, src: Register) {
        let instruction = Instruction::UpdateVariable { name, src };
        self.push_instruction(instruction);
    }

    pub fn load_variable(&mut self, name: String, dst: Register) {
        let instruction = Instruction::LoadVariable { name, dst };
        self.push_instruction(instruction);
    }

    #[must_use]
    pub fn add(&mut self, lhs: Register, rhs: Register) -> Register {
        let dst = self.allocate_register();
        let instruction = Instruction::Add { lhs, rhs, dst };
        self.push_instruction(instruction);
        dst
    }

    #[must_use]
    pub fn subtract(&mut self, lhs: Register, rhs: Register) -> Register {
        let dst = self.allocate_register();
        let instruction = Instruction::Subtract { lhs, rhs, dst };
        self.push_instruction(instruction);
        dst
    }

    #[must_use]
    pub fn multiply(&mut self, lhs: Register, rhs: Register) -> Register {
        let dst = self.allocate_register();
        let instruction = Instruction::Multiply { lhs, rhs, dst };
        self.push_instruction(instruction);
        dst
    }

    #[must_use]
    pub fn divide(&mut self, lhs: Register, rhs: Register) -> Register {
        let dst = self.allocate_register();
        let instruction = Instruction::Divide { lhs, rhs, dst };
        self.push_instruction(instruction);
        dst
    }

    #[must_use]
    pub fn modulo(&mut self, lhs: Register, rhs: Register) -> Register {
        let dst = self.allocate_register();
        let instruction = Instruction::Modulo { lhs, rhs, dst };
        self.push_instruction(instruction);
        dst
    }

    #[must_use]
    pub fn exponentiate(&mut self, lhs: Register, rhs: Register) -> Register {
        let dst = self.allocate_register();
        let instruction = Instruction::Exponentiate { lhs, rhs, dst };
        self.push_instruction(instruction);
        dst
    }

    #[must_use]
    pub fn bitwise_or(&mut self, lhs: Register, rhs: Register) -> Register {
        let dst = self.allocate_register();
        let instruction = Instruction::BitwiseOr { lhs, rhs, dst };
        self.push_instruction(instruction);
        dst
    }

    #[must_use]
    pub fn bitwise_and(&mut self, lhs: Register, rhs: Register) -> Register {
        let dst = self.allocate_register();
        let instruction = Instruction::BitwiseAnd { lhs, rhs, dst };
        self.push_instruction(instruction);
        dst
    }

    #[must_use]
    pub fn bitwise_xor(&mut self, lhs: Register, rhs: Register) -> Register {
        let dst = self.allocate_register();
        let instruction = Instruction::BitwiseXor { lhs, rhs, dst };
        self.push_instruction(instruction);
        dst
    }

    #[must_use]
    pub fn logical_and(&mut self, lhs: Register, rhs: Register) -> Register {
        let dst = self.allocate_register();
        let instruction = Instruction::LogicalAnd { lhs, rhs, dst };
        self.push_instruction(instruction);
        dst
    }

    #[must_use]
    pub fn logical_or(&mut self, lhs: Register, rhs: Register) -> Register {
        let dst = self.allocate_register();
        let instruction = Instruction::LogicalOr { lhs, rhs, dst };
        self.push_instruction(instruction);
        dst
    }

    #[must_use]
    pub fn coalesce(&mut self, lhs: Register, rhs: Register) -> Register {
        let dst = self.allocate_register();
        let instruction = Instruction::Coalesce { lhs, rhs, dst };
        self.push_instruction(instruction);
        dst
    }

    #[must_use]
    pub fn loosely_equal(&mut self, lhs: Register, rhs: Register) -> Register {
        let dst = self.allocate_register();
        let instruction = Instruction::LooselyEqual { lhs, rhs, dst };
        self.push_instruction(instruction);
        dst
    }

    #[must_use]
    pub fn strict_equal(&mut self, lhs: Register, rhs: Register) -> Register {
        let dst = self.allocate_register();
        let instruction = Instruction::StrictEqual { lhs, rhs, dst };
        self.push_instruction(instruction);
        dst
    }

    #[must_use]
    pub fn not_loosely_equal(&mut self, lhs: Register, rhs: Register) -> Register {
        let dst = self.allocate_register();
        let instruction = Instruction::NotLooselyEqual { lhs, rhs, dst };
        self.push_instruction(instruction);
        dst
    }

    #[must_use]
    pub fn strict_not_equal(&mut self, lhs: Register, rhs: Register) -> Register {
        let dst = self.allocate_register();
        let instruction = Instruction::StrictNotEqual { lhs, rhs, dst };
        self.push_instruction(instruction);
        dst
    }

    #[must_use]
    pub fn less_than(&mut self, lhs: Register, rhs: Register) -> Register {
        let dst = self.allocate_register();
        let instruction = Instruction::LessThan { lhs, rhs, dst };
        self.push_instruction(instruction);
        dst
    }

    #[must_use]
    pub fn greater_than(&mut self, lhs: Register, rhs: Register) -> Register {
        let dst = self.allocate_register();
        let instruction = Instruction::GreaterThan { lhs, rhs, dst };
        self.push_instruction(instruction);
        dst
    }

    #[must_use]
    pub fn less_than_or_equal(&mut self, lhs: Register, rhs: Register) -> Register {
        let dst = self.allocate_register();
        let instruction = Instruction::LessThanOrEqual { lhs, rhs, dst };
        self.push_instruction(instruction);
        dst
    }

    #[must_use]
    pub fn greater_than_or_equal(&mut self, lhs: Register, rhs: Register) -> Register {
        let dst = self.allocate_register();
        let instruction = Instruction::GreaterThanOrEqual { lhs, rhs, dst };
        self.push_instruction(instruction);
        dst
    }

    #[must_use]
    pub fn shift_left(&mut self, lhs: Register, rhs: Register) -> Register {
        let dst = self.allocate_register();
        let instruction = Instruction::ShiftLeft { lhs, rhs, dst };
        self.push_instruction(instruction);
        dst
    }

    #[must_use]
    pub fn shift_right(&mut self, lhs: Register, rhs: Register) -> Register {
        let dst = self.allocate_register();
        let instruction = Instruction::ShiftRight { lhs, rhs, dst };
        self.push_instruction(instruction);
        dst
    }

    #[must_use]
    pub fn shift_right_zeros(&mut self, lhs: Register, rhs: Register) -> Register {
        let dst = self.allocate_register();
        let instruction = Instruction::ShiftRightZeros { lhs, rhs, dst };
        self.push_instruction(instruction);
        dst
    }

    pub fn throw(&mut self, value: Register) {
        let instruction = Instruction::Throw { value };
        self.push_instruction(instruction);
    }

    pub fn unconditionally_jump_to(&mut self, branch_to: usize) {
        self.basic_block.exit = BasicBlockExit::GoTo(branch_to);
    }

    pub fn branch_if(&mut self, branch_on: Register, if_true: usize, if_false: usize) {
        self.basic_block.exit = BasicBlockExit::Branch {
            branch_on,
            if_true,
            if_false,
        }
    }

    pub fn create_data_property_or_throw(
        &mut self,
        object: Register,
        property_key: object::PropertyKey,
        property_value: Register,
    ) {
        let instruction = Instruction::CreateDataPropertyOrThrow {
            object,
            property_key,
            property_value,
        };
        self.push_instruction(instruction);
    }

    #[must_use]
    /// <https://262.ecma-international.org/14.0/#sec-tonumber>
    pub fn to_number(&mut self, src: Register) -> Register {
        let dst = self.allocate_register();
        let instruction = Instruction::ToNumber { src, dst };
        self.push_instruction(instruction);

        dst
    }
}
