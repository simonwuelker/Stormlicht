use crate::{
    bytecode::{self, CompileToBytecode},
    parser::{
        identifiers::Identifier,
        tokenization::{Punctuator, SkipLineTerminators, Token, Tokenizer},
        SyntaxError,
    },
};

use super::{parse_primary_expression, Expression};

#[derive(Clone, Debug)]
pub struct MemberExpression {
    /// The element whose member is being accessed
    base: Box<Expression>,
    member: Member,
}

#[derive(Clone, Debug)]
pub enum Member {
    /// `foo.bar`
    Identifier(Identifier),

    /// `foo[bar]`
    Bracket(Box<Expression>),
}

impl MemberExpression {
    pub fn parse<const YIELD: bool, const AWAIT: bool>(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Result<Expression, SyntaxError> {
        let base = parse_primary_expression::<YIELD, AWAIT>(tokenizer)?;
        let next_token = tokenizer.peek(0, SkipLineTerminators::Yes)?;

        let member_expression = match next_token {
            Some(Token::Punctuator(Punctuator::BracketOpen)) => {
                let _ = tokenizer.next(SkipLineTerminators::Yes)?;
                let member_access_expression = Expression::parse::<true, YIELD, AWAIT>(tokenizer)?;
                tokenizer.expect_punctuator(Punctuator::BracketClose)?;

                let member = Member::Bracket(Box::new(member_access_expression));

                Self {
                    base: Box::new(base),
                    member,
                }
                .into()
            },
            Some(Token::Punctuator(Punctuator::Dot)) => {
                let _ = tokenizer.next(SkipLineTerminators::Yes)?;
                let member_name = Identifier::parse(tokenizer)?;

                let member = Member::Identifier(member_name);

                Self {
                    base: Box::new(base),
                    member,
                }
                .into()
            },
            _ => base,
        };

        Ok(member_expression)
    }
}

impl CompileToBytecode for MemberExpression {
    type Result = bytecode::Register;

    fn compile(&self, builder: &mut bytecode::ProgramBuilder) -> Self::Result {
        _ = builder;
        _ = self.base;

        match &self.member {
            Member::Identifier(ident) => _ = ident,
            Member::Bracket(bracket) => _ = bracket,
        }
        todo!("compile member expression")
    }
}
