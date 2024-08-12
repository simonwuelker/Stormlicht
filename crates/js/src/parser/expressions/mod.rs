//! <https://262.ecma-international.org/14.0/#sec-ecmascript-language-expressions>

pub mod assignment_expression;
pub mod binary_expression;
pub mod call;
pub mod conditional;
pub mod left_hand_side_expression;
pub mod member;
pub mod object;
pub mod short_circuit;
pub mod unary_expression;
pub mod update_expression;

pub use assignment_expression::AssignmentExpression;
pub use binary_expression::BinaryExpression;
pub use call::CallExpression;
pub use conditional::ConditionalExpression;
pub use left_hand_side_expression::NewExpression;
pub use member::MemberExpression;
pub use unary_expression::UnaryExpression;
pub use update_expression::UpdateExpression;

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
    Unary(UnaryExpression),
    Update(UpdateExpression),
    IdentifierReference(String),
    New(NewExpression),
    Assignment(AssignmentExpression),
    ConditionalExpression(ConditionalExpression),
    Member(MemberExpression),
    Call(CallExpression),
}

/// <https://262.ecma-international.org/14.0/#prod-PrimaryExpression>
fn parse_primary_expression<const YIELD: bool, const AWAIT: bool>(
    tokenizer: &mut Tokenizer<'_>,
) -> Result<Expression, SyntaxError> {
    let Some(next_token) = tokenizer.peek(0, SkipLineTerminators::Yes)? else {
        return Err(tokenizer.syntax_error("expected more tokens"));
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
            let s = s.clone();
            tokenizer.advance(1);
            Literal::StringLiteral(s).into()
        },
        Token::Identifier(_) => {
            let identifier_reference = parse_identifier_reference::<YIELD, AWAIT>(tokenizer)?;
            Expression::IdentifierReference(identifier_reference)
        },
        Token::Punctuator(Punctuator::CurlyBraceOpen) => {
            let object_literal = ObjectLiteral::parse::<YIELD, AWAIT>(tokenizer)?;
            object_literal.into()
        },
        _ => return Err(tokenizer.syntax_error("failed to parse primary expression")),
    };

    Ok(primary_expression)
}

impl Expression {
    pub fn parse<const IN: bool, const YIELD: bool, const AWAIT: bool>(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Result<Self, SyntaxError> {
        AssignmentExpression::parse::<IN, YIELD, AWAIT>(tokenizer)
    }
}

impl CompileToBytecode for Expression {
    type Result = bytecode::Register;

    fn compile(&self, builder: &mut bytecode::ProgramBuilder) -> Self::Result {
        match self {
            Self::This => todo!(),
            Self::Assignment(assignment_expression) => assignment_expression.compile(builder),
            Self::Binary(binary_expression) => binary_expression.compile(builder),
            Self::Call(call_expression) => call_expression.compile(builder),
            Self::ConditionalExpression(conditional_expression) => {
                conditional_expression.compile(builder)
            },
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
            Self::New(new_expression) => new_expression.compile(builder),
            Self::Member(member_expression) => member_expression.compile(builder),
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

impl From<AssignmentExpression> for Expression {
    fn from(value: AssignmentExpression) -> Self {
        Self::Assignment(value)
    }
}

impl From<ConditionalExpression> for Expression {
    fn from(value: ConditionalExpression) -> Self {
        Self::ConditionalExpression(value)
    }
}

impl From<MemberExpression> for Expression {
    fn from(value: MemberExpression) -> Self {
        Self::Member(value)
    }
}

impl From<CallExpression> for Expression {
    fn from(value: CallExpression) -> Self {
        Self::Call(value)
    }
}
