use super::{Instruction, Register};

#[derive(Clone, Debug, Default)]
pub struct BasicBlock {
    pub registers_required: usize,
    pub instructions: Vec<Instruction>,
    pub exit: BasicBlockExit,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum BasicBlockExit {
    /// Terminate program execution
    #[default]
    Terminate,

    /// Execute either `if_true` or `if_false`, depending on `branch_on`
    Branch {
        branch_on: Register,
        if_true: usize,
        if_false: usize,
    },

    /// Unconditionally execute another basic block
    GoTo(usize),
}

#[derive(Clone, Debug, Default)]
pub struct Program {
    pub(crate) basic_blocks: Vec<BasicBlock>,
}
