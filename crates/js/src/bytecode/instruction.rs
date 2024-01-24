use super::Register;
use crate::Value;

#[derive(Clone, Copy, Debug)]
pub struct VariableHandle(usize);

impl VariableHandle {
    #[must_use]
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    #[must_use]
    pub const fn index(&self) -> usize {
        self.0
    }
}

#[derive(Clone, Debug)]
pub enum Instruction {
    LoadImmediate {
        destination: Register,
        immediate: Value,
    },
    CreateVariable {
        name: String,
    },
    UpdateVariable {
        name: String,
        src: Register,
    },
    LoadVariable {
        name: String,
        dst: Register,
    },
    Add {
        lhs: Register,
        rhs: Register,
        dst: Register,
    },
    Subtract {
        lhs: Register,
        rhs: Register,
        dst: Register,
    },
    Multiply {
        lhs: Register,
        rhs: Register,
        dst: Register,
    },
    Divide {
        lhs: Register,
        rhs: Register,
        dst: Register,
    },
    Modulo {
        lhs: Register,
        rhs: Register,
        dst: Register,
    },
    BitwiseOr {
        lhs: Register,
        rhs: Register,
        dst: Register,
    },
    BitwiseAnd {
        lhs: Register,
        rhs: Register,
        dst: Register,
    },
    BitwiseXor {
        lhs: Register,
        rhs: Register,
        dst: Register,
    },
    LogicalAnd {
        lhs: Register,
        rhs: Register,
        dst: Register,
    },
    LogicalOr {
        lhs: Register,
        rhs: Register,
        dst: Register,
    },
    LooselyEqual {
        lhs: Register,
        rhs: Register,
        dst: Register,
    },
    StrictEqual {
        lhs: Register,
        rhs: Register,
        dst: Register,
    },
    NotLooselyEqual {
        lhs: Register,
        rhs: Register,
        dst: Register,
    },
    StrictNotEqual {
        lhs: Register,
        rhs: Register,
        dst: Register,
    },
    LessThan {
        lhs: Register,
        rhs: Register,
        dst: Register,
    },
    GreaterThan {
        lhs: Register,
        rhs: Register,
        dst: Register,
    },
    LessThanOrEqual {
        lhs: Register,
        rhs: Register,
        dst: Register,
    },
    GreaterThanOrEqual {
        lhs: Register,
        rhs: Register,
        dst: Register,
    },
    ShiftLeft {
        lhs: Register,
        rhs: Register,
        dst: Register,
    },
    ShiftRight {
        lhs: Register,
        rhs: Register,
        dst: Register,
    },
    ShiftRightZeros {
        lhs: Register,
        rhs: Register,
        dst: Register,
    },
}
