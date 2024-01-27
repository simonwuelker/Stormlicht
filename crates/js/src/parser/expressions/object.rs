//! <https://262.ecma-international.org/14.0/#sec-object-initializer>

use crate::{
    bytecode::{self, CompileToBytecode},
    parser::{
        identifiers::parse_identifier_reference, tokenizer::Punctuator, SyntaxError, Tokenizer,
    },
};

/// <https://262.ecma-international.org/14.0/#prod-ObjectLiteral>
#[derive(Clone, Debug)]
pub struct ObjectLiteral {
    property_definitions: Vec<PropertyDefinition>,
}

/// <https://262.ecma-international.org/14.0/#prod-PropertyDefinition>
#[derive(Clone, Debug)]
enum PropertyDefinition {
    IdentifierRef(String),
}

impl PropertyDefinition {
    /// <https://262.ecma-international.org/14.0/#prod-PropertyDefinition>
    pub fn parse<const YIELD: bool, const AWAIT: bool>(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Result<Self, SyntaxError> {
        let property_definition = if let Ok(identifier_reference) =
            parse_identifier_reference::<YIELD, AWAIT>(tokenizer)
        {
            Self::IdentifierRef(identifier_reference)
        } else {
            return Err(tokenizer.syntax_error());
        };

        Ok(property_definition)
    }
}

impl ObjectLiteral {
    /// <https://262.ecma-international.org/14.0/#prod-ObjectLiteral>
    pub fn parse<const YIELD: bool, const AWAIT: bool>(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Result<Self, SyntaxError> {
        tokenizer.expect_punctuator(Punctuator::CurlyBraceOpen)?;
        let property_definitions =
            tokenizer.parse_comma_separated_list(PropertyDefinition::parse::<YIELD, AWAIT>);
        tokenizer.expect_punctuator(Punctuator::CurlyBraceClose)?;

        let object_literal = Self {
            property_definitions,
        };
        Ok(object_literal)
    }
}

impl CompileToBytecode for ObjectLiteral {
    type Result = bytecode::Register;

    fn compile(&self, builder: &mut bytecode::ProgramBuilder) -> Self::Result {
        _ = builder;
        todo!()
    }
}
