use crate::bytecode::{self, CompileToBytecode};

use super::{
    identifiers::parse_identifier_reference, literals::Literal, tokenizer::Punctuator, SyntaxError,
    Tokenizer,
};

#[derive(Clone, Debug)]
pub enum Expression {
    This,
    Literal(Literal),
    Binary(BinaryExpression),
    IdentifierReference(String),
    New(NewExpression),
}

#[derive(Clone, Debug)]
pub struct BinaryExpression {
    pub op: BinaryOp,
    pub lhs: Box<Expression>,
    pub rhs: Box<Expression>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryOp {
    Arithmetic(ArithmeticOp),
    Logical(LogicalOp),
    Bitwise(BitwiseOp),
    Equality(EqualityOp),
    Relational(RelationalOp),
    Shift(ShiftOp),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArithmeticOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogicalOp {
    Or,
    And,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BitwiseOp {
    Or,
    And,
    Xor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EqualityOp {
    Equal,
    NotEqual,
    StrictEqual,
    StrictNotEqual,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelationalOp {
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShiftOp {
    ShiftLeft,
    ShiftRight,
    ShiftRightZeros,
}

macro_rules! binary_op {
    ($docs: expr, $name: ident<$(const $const_ident: ident:$const_type:ty,)*>, $next: path, $($symbol: pat => $op: path,)*) => {
        #[doc = $docs]
        fn $name<$(const $const_ident: $const_type,)*>(
            tokenizer: &mut Tokenizer<'_>,
        ) -> Result<Expression, SyntaxError> {
            let mut expression: Expression = $next(tokenizer)?.into();

            let parse_or_term = |tokenizer: &mut Tokenizer<'_>| {
                let punctuator = tokenizer.attempt(Tokenizer::consume_punctuator)?;

                let operator = match punctuator {
                    $($symbol => $op,)*
                    _ => return Err(tokenizer.syntax_error()),
                };

                let rhs = tokenizer.attempt($next)?;
                Ok((operator, rhs))
            };

            while let Ok((operator, rhs)) = tokenizer.attempt(parse_or_term) {
                expression = BinaryExpression {
                    op: operator.into(),
                    lhs: Box::new(expression),
                    rhs: Box::new(rhs.into()),
                }
                .into();
            }

            Ok(expression)
        }
    };
}

binary_op!(
    "<https://262.ecma-international.org/14.0/#prod-LogicalORExpression>",
    parse_logical_or_expression<const IN: bool, const YIELD: bool, const AWAIT: bool,>,
    parse_logical_and_expression::<IN, YIELD, AWAIT>,
    Punctuator::DoubleVerticalBar => LogicalOp::Or,
);

binary_op!(
    "<https://262.ecma-international.org/14.0/#prod-LogicalANDExpression>",
    parse_logical_and_expression<const IN: bool, const YIELD: bool, const AWAIT: bool,>,
    parse_bitwise_or_expression::<IN, YIELD, AWAIT>,
    Punctuator::DoubleAmpersand => LogicalOp::And,
);

binary_op!(
    "<https://262.ecma-international.org/14.0/#prod-BitwiseORExpression>",
    parse_bitwise_or_expression<const IN: bool, const YIELD: bool, const AWAIT: bool,>,
    parse_bitwise_xor_expression::<IN, YIELD, AWAIT>,
    Punctuator::VerticalBar => BitwiseOp::Or,
);

binary_op!(
    "<https://262.ecma-international.org/14.0/#prod-BitwiseXORExpression>",
    parse_bitwise_xor_expression<const IN: bool, const YIELD: bool, const AWAIT: bool,>,
    parse_bitwise_and_expression::<IN, YIELD, AWAIT>,
    Punctuator::Caret => BitwiseOp::Xor,
);

binary_op!(
    "<https://262.ecma-international.org/14.0/#prod-BitwiseANDExpression>",
    parse_bitwise_and_expression<const IN: bool, const YIELD: bool, const AWAIT: bool,>,
    parse_equality_expression::<IN, YIELD, AWAIT>,
    Punctuator::Ampersand => BitwiseOp::And,
);

binary_op!(
    "<https://262.ecma-international.org/14.0/#prod-EqualityExpression>",
    parse_equality_expression<const IN: bool, const YIELD: bool, const AWAIT: bool,>,
    parse_relational_expression::<IN, YIELD, AWAIT>,
    Punctuator::DoubleEqual => EqualityOp::Equal,
    Punctuator::TripleEqual => EqualityOp::StrictEqual,
    Punctuator::ExclamationMarkEqual => EqualityOp::NotEqual,
    Punctuator::ExclamationMarkDoubleEqual => EqualityOp::StrictEqual,
);

binary_op!(
    "<https://262.ecma-international.org/14.0/#prod-RelationalExpression>",
    parse_relational_expression<const IN: bool, const YIELD: bool, const AWAIT: bool,>,
    parse_shift_expression::<YIELD, AWAIT>,
    Punctuator::LessThan => RelationalOp::LessThan,
    Punctuator::GreaterThan => RelationalOp::GreaterThan,
    Punctuator::LessThanEqual => RelationalOp::LessThanOrEqual,
    Punctuator::GreaterThanEqual => RelationalOp::GreaterThanOrEqual,
);

binary_op!(
    "<https://262.ecma-international.org/14.0/#prod-RelationalExpression>",
    parse_shift_expression<const YIELD: bool, const AWAIT: bool,>,
    parse_additive_expression::<YIELD, AWAIT>,
    Punctuator::DoubleLessThan => ShiftOp::ShiftLeft,
    Punctuator::DoubleGreaterThan => ShiftOp::ShiftRight,
    Punctuator::DoubleGreaterThanEqual => ShiftOp::ShiftRightZeros,
);

binary_op!(
    "<https://262.ecma-international.org/14.0/#prod-AdditiveExpression>",
    parse_additive_expression<const YIELD: bool, const AWAIT: bool,>,
    parse_multiplicative_expression::<YIELD, AWAIT>,
    Punctuator::Plus => ArithmeticOp::Add,
    Punctuator::Minus => ArithmeticOp::Subtract,
);

binary_op!(
    "<https://262.ecma-international.org/14.0/#prod-MultiplicativeExpression>",
    parse_multiplicative_expression<const YIELD: bool, const AWAIT: bool,>,
    parse_primary_expression::<YIELD, AWAIT>,
    Punctuator::Asterisk => ArithmeticOp::Multiply,
    Punctuator::Slash => ArithmeticOp::Divide,
    Punctuator::Percent => ArithmeticOp::Modulo,
);

/// <https://262.ecma-international.org/14.0/#prod-LeftHandSideExpression>
#[derive(Clone, Debug)]
pub enum LeftHandSideExpression {
    NewExpression(NewExpression),
    CallExpression,
    OptionalExpression,
}

/// <https://262.ecma-international.org/14.0/#prod-NewExpression>
#[derive(Clone, Debug)]
pub struct NewExpression {
    /// The number of `new` keywords before the expression
    pub nest_level: usize,
    pub expression: Box<Expression>,
}

/// <https://262.ecma-international.org/14.0/#prod-PrimaryExpression>
fn parse_primary_expression<const YIELD: bool, const AWAIT: bool>(
    tokenizer: &mut Tokenizer<'_>,
) -> Result<Expression, SyntaxError> {
    if tokenizer
        .attempt(|tokenizer| tokenizer.consume_keyword("this"))
        .is_ok()
    {
        Ok(Expression::This)
    } else if let Ok(identifier) = tokenizer.attempt(parse_identifier_reference::<YIELD, AWAIT>) {
        Ok(Expression::IdentifierReference(identifier))
    } else if let Ok(literal) = Literal::parse(tokenizer) {
        Ok(Expression::Literal(literal))
    } else {
        Err(tokenizer.syntax_error())
    }
}

impl NewExpression {
    /// <https://262.ecma-international.org/14.0/#prod-NewExpression>
    fn parse<const IN: bool, const YIELD: bool, const AWAIT: bool>(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Result<Expression, SyntaxError> {
        let mut nest_level = 0;

        while matches!(
            tokenizer.attempt(Tokenizer::consume_identifier).as_deref(),
            Ok("new")
        ) {
            nest_level += 1;
        }

        // FIXME: This should be a MemberExpression instead of a PrimaryExpression
        let member_expression = parse_primary_expression::<YIELD, AWAIT>(tokenizer)?;

        if nest_level == 0 {
            Ok(member_expression)
        } else {
            Ok(Self {
                nest_level,
                expression: Box::new(member_expression),
            }
            .into())
        }
    }
}

impl Expression {
    pub fn parse<const IN: bool, const YIELD: bool, const AWAIT: bool>(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Result<Self, SyntaxError> {
        parse_logical_or_expression::<IN, YIELD, AWAIT>(tokenizer)
    }
}

impl CompileToBytecode for Expression {
    type Result = bytecode::Register;

    fn compile(&self, builder: &mut bytecode::Builder) -> Self::Result {
        match self {
            Self::Binary(binary_expression) => {
                let lhs = binary_expression.lhs.compile(builder);
                let rhs = binary_expression.rhs.compile(builder);

                let dst = match binary_expression.op {
                    BinaryOp::Arithmetic(ArithmeticOp::Add) => builder.add(lhs, rhs),
                    BinaryOp::Arithmetic(ArithmeticOp::Subtract) => builder.subtract(lhs, rhs),
                    BinaryOp::Arithmetic(ArithmeticOp::Multiply) => builder.multiply(lhs, rhs),
                    BinaryOp::Arithmetic(ArithmeticOp::Divide) => builder.divide(lhs, rhs),
                    BinaryOp::Arithmetic(ArithmeticOp::Modulo) => builder.modulo(lhs, rhs),
                    BinaryOp::Bitwise(BitwiseOp::And) => builder.bitwise_and(lhs, rhs),
                    BinaryOp::Bitwise(BitwiseOp::Or) => builder.bitwise_or(lhs, rhs),
                    BinaryOp::Bitwise(BitwiseOp::Xor) => builder.bitwise_xor(lhs, rhs),
                    BinaryOp::Logical(LogicalOp::And) => builder.logical_and(lhs, rhs),
                    BinaryOp::Logical(LogicalOp::Or) => builder.logical_or(lhs, rhs),
                    BinaryOp::Equality(EqualityOp::Equal) => builder.equal(lhs, rhs),
                    BinaryOp::Equality(EqualityOp::NotEqual) => builder.equal(lhs, rhs),
                    BinaryOp::Equality(EqualityOp::StrictEqual) => builder.strict_equal(lhs, rhs),
                    BinaryOp::Equality(EqualityOp::StrictNotEqual) => {
                        builder.strict_not_equal(lhs, rhs)
                    },
                    BinaryOp::Relational(RelationalOp::LessThan) => builder.less_than(lhs, rhs),
                    BinaryOp::Relational(RelationalOp::GreaterThan) => {
                        builder.greater_than(lhs, rhs)
                    },
                    BinaryOp::Relational(RelationalOp::LessThanOrEqual) => {
                        builder.less_than_or_equal(lhs, rhs)
                    },
                    BinaryOp::Relational(RelationalOp::GreaterThanOrEqual) => {
                        builder.greater_than_or_equal(lhs, rhs)
                    },
                    BinaryOp::Shift(ShiftOp::ShiftLeft) => builder.shift_left(lhs, rhs),
                    BinaryOp::Shift(ShiftOp::ShiftRight) => builder.shift_right(lhs, rhs),
                    BinaryOp::Shift(ShiftOp::ShiftRightZeros) => {
                        builder.shift_right_zeros(lhs, rhs)
                    },
                };

                dst
            },
            Self::IdentifierReference(_) => todo!(),
            Self::Literal(literal) => builder.allocate_register_with_value(literal.clone().into()),
            Self::New(_) => todo!(),
            Self::This => todo!(),
        }
    }
}

impl From<Literal> for Expression {
    fn from(value: Literal) -> Self {
        Self::Literal(value)
    }
}

impl From<BinaryExpression> for Expression {
    fn from(value: BinaryExpression) -> Self {
        Self::Binary(value)
    }
}

impl From<NewExpression> for Expression {
    fn from(value: NewExpression) -> Self {
        Self::New(value)
    }
}

impl From<ArithmeticOp> for BinaryOp {
    fn from(value: ArithmeticOp) -> Self {
        Self::Arithmetic(value)
    }
}

impl From<LogicalOp> for BinaryOp {
    fn from(value: LogicalOp) -> Self {
        Self::Logical(value)
    }
}

impl From<BitwiseOp> for BinaryOp {
    fn from(value: BitwiseOp) -> Self {
        Self::Bitwise(value)
    }
}

impl From<EqualityOp> for BinaryOp {
    fn from(value: EqualityOp) -> Self {
        Self::Equality(value)
    }
}

impl From<RelationalOp> for BinaryOp {
    fn from(value: RelationalOp) -> Self {
        Self::Relational(value)
    }
}

impl From<ShiftOp> for BinaryOp {
    fn from(value: ShiftOp) -> Self {
        Self::Shift(value)
    }
}
