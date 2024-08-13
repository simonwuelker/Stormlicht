//! <https://262.ecma-international.org/14.0/#sec-assignment-operators>

use crate::parser::{
    tokenization::{Punctuator, SkipLineTerminators, Token, Tokenizer},
    SyntaxError,
};

use super::{ConditionalExpression, Expression};

/// <https://262.ecma-international.org/14.0/#prod-AssignmentExpression>
#[derive(Clone, Debug)]
pub struct AssignmentExpression {
    lhs: Box<AssignmentTarget>,
    operator: AssignmentOp,
    rhs: Box<Expression>,
}

#[derive(Clone, Debug)]
enum AssignmentTarget {
    IdentifierRef(String),
    // TODO: Variants are missing here
}

impl AssignmentExpression {
    /// <https://262.ecma-international.org/14.0/#prod-AssignmentExpression>
    pub fn parse<const IN: bool, const YIELD: bool, const AWAIT: bool>(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Result<Expression, SyntaxError> {
        let Some(next_token) = tokenizer.peek(0, SkipLineTerminators::Yes)? else {
            return Err(tokenizer.syntax_error("expected more tokens"));
        };

        match next_token {
            Token::Identifier(ident) if YIELD && ident == "yield" => {
                todo!("implement parsing yield expression");
            },
            _ => {},
        }

        // This works because every LeftHandSideExpression is also a valid ConditionalExpression
        let conditional_expression = ConditionalExpression::parse::<IN, YIELD, AWAIT>(tokenizer)?;

        let next_token = tokenizer.peek(0, SkipLineTerminators::Yes)?;

        if let Some(operator) = next_token.and_then(AssignmentOp::from_token) {
            tokenizer.advance(1);

            let Some(lhs) =
                AssignmentTarget::from_expression(conditional_expression, tokenizer.is_strict())
            else {
                return Err(tokenizer.syntax_error("expression is not a valid assignment target"));
            };

            let rhs = AssignmentExpression::parse::<IN, YIELD, AWAIT>(tokenizer)?;

            let assignment_expression = Self {
                lhs: Box::new(lhs),
                operator,
                rhs: Box::new(rhs),
            };

            Ok(assignment_expression.into())
        } else {
            Ok(conditional_expression.into())
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum AssignmentOp {
    /// The `=` operator
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Assignment)
    Assignment,

    /// The `*=` operator
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Multiplication_assignment)
    MultiplicationAssignment,

    /// The `/=` operator
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Division_assignment)
    DivisionAssignment,

    /// The `%=` operator
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Remainder_assignment)
    RemainderAssignment,

    /// The `+=` operator
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Addition_assignment)
    AdditionAssignment,

    /// The `-=` operator
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Subtraction_assignment)
    SubtractionAssignment,

    /// The `**=` operator
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Exponentiation_assignment)
    ExponentiationAssignment,

    /// The `<<=` operator
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Left_shift_assignment)
    LeftShiftAssignment,

    /// The `>>=` operator
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Right_shift_assignment)
    RightShiftAssignment,

    /// The `>>>=` operator
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Unsigned_right_shift_assignment)
    UnsignedRightShiftAssignment,

    /// The `&=` operator
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Bitwise_AND_assignment)
    BitwiseAndAssignment,

    /// The `^=` operator
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Bitwise_XOR_assignment)
    BitwiseXorAssignment,

    /// The `|=` operator
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Bitwise_OR_assignment)
    BitwiseOrAssignment,

    /// The `&&=` operator
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Logical_AND_assignment)
    LogicalAndAssignment,

    /// The `||=` operator
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Logical_OR_assignment)
    LogicalOrAssignment,

    /// The `??=` operator
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Nullish_coalescing_assignment)
    NullishCoalescingAssignment,
}

impl AssignmentOp {
    #[must_use]
    fn from_token(token: &Token) -> Option<Self> {
        let operator = match token {
            Token::Punctuator(Punctuator::Equal) => AssignmentOp::Assignment,
            Token::Punctuator(Punctuator::AsteriskEqual) => AssignmentOp::MultiplicationAssignment,
            Token::Punctuator(Punctuator::SlashEqual) => AssignmentOp::DivisionAssignment,
            Token::Punctuator(Punctuator::PercentEqual) => AssignmentOp::RemainderAssignment,
            Token::Punctuator(Punctuator::PlusEqual) => AssignmentOp::AdditionAssignment,
            Token::Punctuator(Punctuator::MinusEqual) => AssignmentOp::SubtractionAssignment,
            Token::Punctuator(Punctuator::DoubleAsteriskEqual) => {
                AssignmentOp::ExponentiationAssignment
            },
            Token::Punctuator(Punctuator::DoubleLessThanEqual) => AssignmentOp::LeftShiftAssignment,
            Token::Punctuator(Punctuator::DoubleGreaterThanEqual) => {
                AssignmentOp::RightShiftAssignment
            },
            Token::Punctuator(Punctuator::TripleGreaterThanEqual) => {
                AssignmentOp::UnsignedRightShiftAssignment
            },
            Token::Punctuator(Punctuator::AmpersandEqual) => AssignmentOp::BitwiseAndAssignment,
            Token::Punctuator(Punctuator::CaretEqual) => AssignmentOp::BitwiseXorAssignment,
            Token::Punctuator(Punctuator::VerticalBarEqual) => AssignmentOp::BitwiseOrAssignment,
            Token::Punctuator(Punctuator::DoubleAmpersandEqual) => {
                AssignmentOp::LogicalAndAssignment
            },
            Token::Punctuator(Punctuator::DoubleVerticalBarEqual) => {
                AssignmentOp::LogicalOrAssignment
            },
            Token::Punctuator(Punctuator::DoubleQuestionMarkEqual) => {
                AssignmentOp::NullishCoalescingAssignment
            },
            _ => return None,
        };

        Some(operator)
    }
}

impl AssignmentTarget {
    #[must_use]
    fn from_expression(expression: Expression, is_strict_mode: bool) -> Option<Self> {
        let assignment_target = match expression {
            Expression::IdentifierReference(identifier) => {
                // 1. If this IdentifierReference is contained in strict mode code and
                //    StringValue of Identifier is either "eval" or "arguments", return invalid.
                if is_strict_mode && matches!(identifier.as_str(), "eval" | "arguments") {
                    return None;
                }

                Self::IdentifierRef(identifier)
            },
            _ => return None,
        };

        Some(assignment_target)
    }
}
