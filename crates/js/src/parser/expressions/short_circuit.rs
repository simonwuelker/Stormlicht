//! <https://262.ecma-international.org/14.0/#prod-ShortCircuitExpression>

use crate::parser::{
    tokenization::{Punctuator, SkipLineTerminators, Token, Tokenizer},
    SyntaxError,
};

use super::{
    binary_expression::{parse_bitwise_or_expression, parse_logical_and_expression, LogicalOp},
    BinaryExpression, Expression,
};

pub fn parse_short_circuit_expression<const IN: bool, const YIELD: bool, const AWAIT: bool>(
    tokenizer: &mut Tokenizer<'_>,
) -> Result<Expression, SyntaxError> {
    // This production is rather complicated, so we effectively sidestep the contained LogicalOr/LogicalAnd productions
    // and just connect BitwiseOr expressions ourselves

    let mut expression = parse_bitwise_or_expression::<IN, YIELD, AWAIT>(tokenizer)?;

    if let Some(Token::Punctuator(punct)) = tokenizer.peek(0, SkipLineTerminators::Yes)? {
        match punct {
            Punctuator::DoubleQuestionMark => {
                // We're in a coalesce expression
                tokenizer.advance(1);

                return parse_coalesce_expression_where_first_element_has_been_parsed::<
                    IN,
                    YIELD,
                    AWAIT,
                >(tokenizer, expression);
            },
            Punctuator::DoubleAmpersand => {
                // We're in a logical AND expression, potentially nested in another logical OR expression
                tokenizer.advance(1);

                expression = parse_logical_and_expression_where_first_element_has_been_parsed::<
                    IN,
                    YIELD,
                    AWAIT,
                >(tokenizer, expression)?;

                if let Some(Token::Punctuator(Punctuator::DoubleVerticalBar)) =
                    tokenizer.peek(0, SkipLineTerminators::Yes)?
                {
                    tokenizer.advance(1);

                    expression = parse_logical_or_expression_where_first_element_has_been_parsed::<
                        IN,
                        YIELD,
                        AWAIT,
                    >(tokenizer, expression)?;
                }
            },
            Punctuator::DoubleVerticalBar => {
                // We're in a logical OR expression
                tokenizer.advance(1);

                expression = parse_logical_or_expression_where_first_element_has_been_parsed::<
                    IN,
                    YIELD,
                    AWAIT,
                >(tokenizer, expression)?;
            },
            _ => {},
        }
    }

    Ok(expression)
}

/// Create a parsing function for a chain of operands that assumes that the first element/operand have already
/// been parsed.
///
/// This means that assuming an input like `foo && bar && baz`, the parser would have the following position when the
/// parser function is called:
/// ```text, ignore
/// foo && bar && baz
///        ^- here
/// ```
///
/// Due to the operator being parsed, it means that *at least* one more operand must follow.
macro_rules! parse_binary_op_where_first_element_has_been_parsed {
    ($name: ident<$(const $const_ident: ident:$const_type:ty,)*>, $op: expr, $punct: expr, $next: path) => {
        fn $name<$(const $const_ident: $const_type,)*>(
            tokenizer: &mut Tokenizer<'_>,
            first_part: Expression,
        ) -> Result<Expression, SyntaxError> {
            let rhs = $next(tokenizer)?;
            let mut expression = BinaryExpression {
                op: $op.into(),
                lhs: Box::new(first_part),
                rhs: Box::new(rhs),
            }
            .into();

            while tokenizer
                .peek(0, SkipLineTerminators::Yes)?
                .is_some_and(|t| t.is_punctuator($punct))
            {
                tokenizer.advance(1);
                let rhs = $next(tokenizer)?;

                expression = BinaryExpression {
                    op: $op.into(),
                    lhs: Box::new(expression),
                    rhs: Box::new(rhs),
                }
                .into();
            }

            Ok(expression)
        }
    };
}

parse_binary_op_where_first_element_has_been_parsed! {
    parse_coalesce_expression_where_first_element_has_been_parsed<const IN: bool, const YIELD: bool, const AWAIT: bool,>,
    LogicalOp::Coalesce,
    Punctuator::DoubleQuestionMark,
    parse_bitwise_or_expression<IN, YIELD, AWAIT>
}

parse_binary_op_where_first_element_has_been_parsed! {
    parse_logical_and_expression_where_first_element_has_been_parsed<const IN: bool, const YIELD: bool, const AWAIT: bool,>,
    LogicalOp::And,
    Punctuator::DoubleAmpersand,
    parse_bitwise_or_expression<IN, YIELD, AWAIT>
}

parse_binary_op_where_first_element_has_been_parsed! {
    parse_logical_or_expression_where_first_element_has_been_parsed<const IN: bool, const YIELD: bool, const AWAIT: bool,>,
    LogicalOp::Or,
    Punctuator::DoubleVerticalBar,
    parse_logical_and_expression<IN, YIELD, AWAIT>
}
