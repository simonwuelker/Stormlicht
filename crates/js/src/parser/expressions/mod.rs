//! <https://262.ecma-international.org/14.0/#sec-ecmascript-language-expressions>

mod binary_expression;
mod left_hand_side_expression;
mod object;
mod unary_expression;
mod update_expression;

pub use binary_expression::BinaryExpression;
pub use left_hand_side_expression::LeftHandSideExpression;
pub use unary_expression::UnaryExpression;
pub use update_expression::UpdateExpression;

use crate::{
    bytecode::{self, CompileToBytecode},
    Number,
};

use self::{left_hand_side_expression::NewExpression, object::ObjectLiteral};

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
    Unary(UnaryExpression),
    Update(UpdateExpression),
    IdentifierReference(String),
    New(NewExpression),
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
            Self::Unary(unary_expression) => unary_expression.compile(builder),
            Self::Update(update_expression) => update_expression.compile(builder),
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

impl From<UnaryExpression> for Expression {
    fn from(value: UnaryExpression) -> Self {
        Self::Unary(value)
    }
}

impl From<UpdateExpression> for Expression {
    fn from(value: UpdateExpression) -> Self {
        Self::Update(value)
    }
}
