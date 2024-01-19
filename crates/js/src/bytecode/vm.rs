use super::Instruction;

#[derive(Clone, Debug, Default)]
pub enum Value {
    /// <https://262.ecma-international.org/14.0/#sec-ecmascript-language-types-undefined-type>
    #[default]
    Undefined,

    /// <https://262.ecma-international.org/14.0/#sec-ecmascript-language-types-null-type>
    Null,

    /// <https://262.ecma-international.org/14.0/#sec-ecmascript-language-types-boolean-type>
    Boolean(bool),

    /// <https://262.ecma-international.org/14.0/#sec-ecmascript-language-types-string-type>
    String(String),
}

#[derive(Clone, Debug)]
pub struct Vm {
    registers: Vec<Value>,
}

impl Vm {
    pub fn execute_basic_block(&mut self, block: &BasicBlock) {
        self.registers
            .resize_with(block.registers_required, Default::default);

        for instruction in &block.instructions {
            self.execute_instruction(instruction);
        }
    }

    fn execute_instruction(&mut self, instruction: &Instruction) {
        match instruction {
            Instruction::LoadImmediate {
                destination,
                immediate,
            } => {
                self.registers[destination.index()] = immediate.clone();
            },
            _ => todo!(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct BasicBlock {
    pub registers_required: usize,
    pub instructions: Vec<Instruction>,
}
