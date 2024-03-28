//! <https://262.ecma-international.org/14.0/#sec-ecmascript-language-expressions>

mod binary_expression;
mod object;

pub use binary_expression::BinaryExpression;

use crate::{
    bytecode::{self, CompileToBytecode},
    Number,
};

use self::object::ObjectLiteral;

use super::{
    identifiers::parse_identifier_reference,
    literals::Literal,
    tokenization::{Punctuator, SkipLineTerminators, Token, Tokenizer},
    SyntaxError,
};

#[derive(Clone, Debug)]
pub enum Expression {
    This,
    Literal(Literal),
    ObjectLiteral(ObjectLiteral),
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
    let Some(next_token) = tokenizer.peek(0, SkipLineTerminators::Yes)? else {
        return Err(tokenizer.syntax_error());
    };

    let primary_expression = match next_token {
        Token::Identifier(ident) if ident == "this" => {
            tokenizer.advance(1);
            Expression::This
        },
        Token::Identifier(ident) if ident == "true" => {
            tokenizer.advance(1);
            Literal::BooleanLiteral(true).into()
        },
        Token::Identifier(ident) if ident == "false" => {
            tokenizer.advance(1);
            Literal::BooleanLiteral(false).into()
        },
        Token::Identifier(ident) if ident == "null" => {
            tokenizer.advance(1);
            Literal::NullLiteral.into()
        },
        Token::NumericLiteral(n) => {
            let n = Number::new(*n as f64);
            tokenizer.advance(1);
            Literal::NumericLiteral(n).into()
        },
        Token::StringLiteral(s) => {
            // FIXME: avoiding a clone here would be nice
            let s = s.clone();
            tokenizer.advance(1);
            Literal::StringLiteral(s.clone()).into()
        },
        Token::Identifier(_) => {
            let identifier_reference = parse_identifier_reference::<YIELD, AWAIT>(tokenizer)?;
            Expression::IdentifierReference(identifier_reference)
        },
        Token::Punctuator(Punctuator::CurlyBraceOpen) => {
            let object_literal = ObjectLiteral::parse::<YIELD, AWAIT>(tokenizer)?;
            object_literal.into()
        },
        _ => return Err(tokenizer.syntax_error()),
    };

    Ok(primary_expression)
}

impl NewExpression {
    /// <https://262.ecma-international.org/14.0/#prod-NewExpression>
    fn parse<const IN: bool, const YIELD: bool, const AWAIT: bool>(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Result<Expression, SyntaxError> {
        let mut nest_level = 0;

        while tokenizer
            .peek(0, SkipLineTerminators::Yes)?
            .is_some_and(|t| t.is_identifier("new"))
        {
            tokenizer.advance(1);
            nest_level += 1;
        }

        // FIXME: This should be a MemberExpression instead of a PrimaryExpression
        let member_expression = parse_primary_expression::<YIELD, AWAIT>(tokenizer)?;

        let new_expression = if nest_level == 0 {
            member_expression
        } else {
            Self {
                nest_level,
                expression: Box::new(member_expression),
            }
            .into()
        };

        Ok(new_expression)
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
            Self::This => todo!(),
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
            Self::ObjectLiteral(object_literal) => object_literal.compile(builder),
            Self::New(_) => todo!(),
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

impl From<ObjectLiteral> for Expression {
    fn from(value: ObjectLiteral) -> Self {
        Self::ObjectLiteral(value)
    }
}
