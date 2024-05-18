//! <https://262.ecma-international.org/14.0/#sec-object-initializer>

use crate::{
    bytecode::{self, CompileToBytecode},
    parser::{
        identifiers::parse_identifier_reference,
        tokenization::{Punctuator, SkipLineTerminators, Token, Tokenizer},
        SyntaxError,
    },
    value::{object::PropertyKey, Object},
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
        let property_definition =
            parse_identifier_reference::<YIELD, AWAIT>(tokenizer).map(Self::IdentifierRef)?;

        Ok(property_definition)
    }
}

impl ObjectLiteral {
    /// <https://262.ecma-international.org/14.0/#prod-ObjectLiteral>
    pub fn parse<const YIELD: bool, const AWAIT: bool>(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Result<Self, SyntaxError> {
        tokenizer.expect_punctuator(Punctuator::CurlyBraceOpen)?;

        let mut property_definitions = vec![];

        while !matches!(
            tokenizer.peek(0, SkipLineTerminators::Yes)?,
            Some(Token::Punctuator(Punctuator::CurlyBraceClose))
        ) {
            let property_definition = PropertyDefinition::parse::<YIELD, AWAIT>(tokenizer)?;
            property_definitions.push(property_definition);

            if let Some(Token::Punctuator(Punctuator::Comma)) =
                tokenizer.peek(0, SkipLineTerminators::Yes)?
            {
                tokenizer.advance(1);
            }
        }

        // Discard the closing brace
        tokenizer.advance(1);

        let object_literal = Self {
            property_definitions,
        };
        Ok(object_literal)
    }
}

impl CompileToBytecode for ObjectLiteral {
    type Result = bytecode::Register;

    fn compile(&self, builder: &mut bytecode::ProgramBuilder) -> Self::Result {
        let object = builder
            .get_current_block()
            .allocate_register_with_value(Object::default().into());

        for property_definition in &self.property_definitions {
            let property_register = property_definition.compile(builder);
            builder.get_current_block().create_data_property_or_throw(
                object,
                PropertyKey::String(property_register.name),
                property_register.value,
            );
        }

        object
    }
}

#[derive(Debug)]
struct PropertyToBeCreated {
    name: String,
    value: bytecode::Register,
}

impl CompileToBytecode for PropertyDefinition {
    type Result = PropertyToBeCreated;

    fn compile(&self, builder: &mut bytecode::ProgramBuilder) -> Self::Result {
        let mut builder = builder.get_current_block();

        match self {
            Self::IdentifierRef(identifier_reference) => {
                let expr_value = builder.allocate_register();
                builder.load_variable(identifier_reference.clone(), expr_value);

                PropertyToBeCreated {
                    name: identifier_reference.clone(),
                    value: expr_value,
                }
            },
        }
    }
}
