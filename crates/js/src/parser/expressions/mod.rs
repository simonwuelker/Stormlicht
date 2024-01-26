//! <https://262.ecma-international.org/14.0/#sec-ecmascript-language-expressions>

mod binary_expression;

pub use binary_expression::BinaryExpression;

use crate::bytecode::{self, CompileToBytecode};

use super::{identifiers::parse_identifier_reference, literals::Literal, SyntaxError, Tokenizer};

#[derive(Clone, Debug)]
pub enum Expression {
    This,
    Literal(Literal),
    Binary(BinaryExpression),
    IdentifierReference(String),
    New(NewExpression),
}

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
        .attempt(|tokenizer| tokenizer.expect_keyword("this"))
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
        binary_expression::parse_logical_or_expression::<IN, YIELD, AWAIT>(tokenizer)
    }
}

impl CompileToBytecode for Expression {
    type Result = bytecode::Register;

    fn compile(&self, builder: &mut bytecode::ProgramBuilder) -> Self::Result {
        match self {
            Self::Binary(binary_expression) => binary_expression.compile(builder),
            Self::IdentifierReference(identifier_reference) => {
                let mut current_block = builder.get_current_block();
                let dst = current_block.allocate_register();
                current_block.load_variable(identifier_reference.clone(), dst);
                dst
            },
            Self::Literal(literal) => builder
                .get_current_block()
                .allocate_register_with_value(literal.clone().into()),
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
