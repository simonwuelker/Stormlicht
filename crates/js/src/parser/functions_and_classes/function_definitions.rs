//! <https://262.ecma-international.org/14.0/#sec-function-definitions>

use crate::{
    bytecode::{self, CompileToBytecode},
    parser::{
        identifiers::parse_binding_identifier, script::Statement, tokenizer::Punctuator,
        SyntaxError, Tokenizer,
    },
};

/// <https://262.ecma-international.org/14.0/#prod-FunctionDeclaration>
#[derive(Clone, Debug)]
pub struct FunctionDeclaration {
    pub identifier: String,
    pub body: Vec<Statement>,
}

impl FunctionDeclaration {
    /// <https://262.ecma-international.org/14.0/#prod-FunctionDeclaration>
    pub fn parse<const YIELD: bool, const AWAIT: bool, const DEFAULT: bool>(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Result<Self, SyntaxError> {
        tokenizer.expect_keyword("function")?;

        let identifier = parse_binding_identifier::<YIELD, AWAIT>(tokenizer)?;
        tokenizer.expect_punctuator(Punctuator::ParenthesisOpen)?;
        tokenizer.expect_punctuator(Punctuator::ParenthesisClose)?;
        tokenizer.expect_punctuator(Punctuator::CurlyBraceOpen)?;
        tokenizer.expect_punctuator(Punctuator::CurlyBraceClose)?;

        let body = vec![];

        let function_declaration = Self { identifier, body };

        Ok(function_declaration)
    }
}

impl CompileToBytecode for FunctionDeclaration {
    fn compile(&self, builder: &mut bytecode::Builder) {
        _ = builder;
        todo!()
    }
}
