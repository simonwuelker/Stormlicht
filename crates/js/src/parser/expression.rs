use super::{literals::Literal, tokenizer::Punctuator, SyntaxError, Tokenizer};

#[derive(Clone, Debug)]
pub enum Expression {
    This,
    Literal(Literal),
    Binary(BinaryExpression),
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArithmeticOp {
    Add,
    Subtract,
    Multiply,
    Divide,
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

macro_rules! binary_op {
    ($docs: expr, $name: ident, $next: path, $op: path, $symbol: pat) => {
        #[doc = $docs]
        fn $name<const IN: bool, const YIELD: bool, const AWAIT: bool>(
            tokenizer: &mut Tokenizer<'_>,
        ) -> Result<Expression, SyntaxError> {
            let mut expression: Expression = $next(tokenizer)?.into();

            let parse_or_term = |tokenizer: &mut Tokenizer<'_>| {
                if !matches!(
                    tokenizer.attempt(Tokenizer::consume_punctuator),
                    Ok(Punctuator::DoubleVerticalBar)
                ) {
                    return Err(tokenizer.syntax_error());
                }

                tokenizer.attempt($next)
            };

            while let Ok(next_term) = tokenizer.attempt(parse_or_term) {
                expression = BinaryExpression {
                    op: $op.into(),
                    lhs: Box::new(expression),
                    rhs: Box::new(next_term.into()),
                }
                .into();
            }

            Ok(expression)
        }
    };
}

binary_op!(
    "<https://262.ecma-international.org/14.0/#prod-LogicalORExpression>",
    parse_logical_or_expression,
    parse_logical_and_expression::<IN, YIELD, AWAIT>,
    LogicalOp::Or,
    Punctuator::DoubleVerticalBar
);

binary_op!(
    "<https://262.ecma-international.org/14.0/#prod-LogicalANDExpression>",
    parse_logical_and_expression,
    parse_bitwise_or_expression::<IN, YIELD, AWAIT>,
    LogicalOp::And,
    Punctuator::DoubleAmpersand
);

binary_op!(
    "<https://262.ecma-international.org/14.0/#prod-BitwiseORExpression>",
    parse_bitwise_or_expression,
    parse_bitwise_xor_expression::<IN, YIELD, AWAIT>,
    BitwiseOp::Or,
    Punctuator::VerticalBar
);

binary_op!(
    "<https://262.ecma-international.org/14.0/#prod-BitwiseXORExpression>",
    parse_bitwise_xor_expression,
    parse_bitwise_and_expression::<IN, YIELD, AWAIT>,
    BitwiseOp::Xor,
    Punctuator::Caret
);

binary_op!(
    "<https://262.ecma-international.org/14.0/#prod-BitwiseANDExpression>",
    parse_bitwise_and_expression,
    NewExpression::parse::<IN, YIELD, AWAIT>,
    BitwiseOp::And,
    Punctuator::Ampersand
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
fn parse_primary_expression<const IN: bool, const YIELD: bool, const AWAIT: bool>(
    tokenizer: &mut Tokenizer<'_>,
) -> Result<Expression, SyntaxError> {
    if matches!(
        tokenizer.attempt(Tokenizer::consume_identifier).as_deref(),
        Ok("this")
    ) {
        Ok(Expression::This)
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
        let member_expression = parse_primary_expression::<IN, YIELD, AWAIT>(tokenizer)?;

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
