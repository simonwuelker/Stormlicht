use crate::parser;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpCode {
    Add,
    Subtract,
    Multiply,
    Divide,
    Exponentiate,
    Modulo,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    LogicalAnd,
    LogicalOr,
    Coalesce,
    LooselyEqual,
    LooselyNotEqual,
    StrictlyEqual,
    StrictlyNotEqual,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
    ShiftLeft,
    ShiftRight,
    ShiftRightZeros,
    Jump(usize),
    JumpIfTrue(usize),
    JumpIfFalse(usize),
}

impl From<parser::BinaryOp> for OpCode {
    fn from(value: parser::BinaryOp) -> Self {
        use parser::{
            ArithmeticOp, BinaryOp, BitwiseOp, EqualityOp, LogicalOp, RelationalOp, ShiftOp,
        };

        match value {
            parser::BinaryOp::Arithmetic(ArithmeticOp::Add) => Self::Add,
            parser::BinaryOp::Arithmetic(ArithmeticOp::Subtract) => Self::Subtract,
            parser::BinaryOp::Arithmetic(ArithmeticOp::Multiply) => Self::Multiply,
            parser::BinaryOp::Arithmetic(ArithmeticOp::Divide) => Self::Divide,
            parser::BinaryOp::Arithmetic(ArithmeticOp::Modulo) => Self::Modulo,
            parser::BinaryOp::Arithmetic(ArithmeticOp::Exponentiation) => Self::Exponentiate,
            parser::BinaryOp::Bitwise(BitwiseOp::And) => Self::BitwiseAnd,
            parser::BinaryOp::Bitwise(BitwiseOp::Or) => Self::BitwiseOr,
            parser::BinaryOp::Bitwise(BitwiseOp::Xor) => Self::BitwiseXor,
            parser::BinaryOp::Logical(LogicalOp::And) => Self::LogicalAnd,
            parser::BinaryOp::Logical(LogicalOp::Or) => Self::LogicalOr,
            BinaryOp::Logical(LogicalOp::Coalesce) => Self::Coalesce,
            parser::BinaryOp::Equality(EqualityOp::Equal) => Self::LooselyEqual,
            parser::BinaryOp::Equality(EqualityOp::NotEqual) => Self::LooselyNotEqual,
            parser::BinaryOp::Equality(EqualityOp::StrictEqual) => Self::StrictlyEqual,
            parser::BinaryOp::Equality(EqualityOp::StrictNotEqual) => Self::StrictlyNotEqual,
            BinaryOp::Relational(RelationalOp::LessThan) => Self::LessThan,
            BinaryOp::Relational(RelationalOp::GreaterThan) => Self::GreaterThan,
            BinaryOp::Relational(RelationalOp::LessThanOrEqual) => Self::LessThanOrEqual,
            BinaryOp::Relational(RelationalOp::GreaterThanOrEqual) => Self::GreaterThanOrEqual,
            BinaryOp::Shift(ShiftOp::ShiftLeft) => Self::ShiftLeft,
            BinaryOp::Shift(ShiftOp::ShiftRight) => Self::ShiftRight,
            BinaryOp::Shift(ShiftOp::ShiftRightZeros) => Self::ShiftRightZeros,
        }
    }
}
