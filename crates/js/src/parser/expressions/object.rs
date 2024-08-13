//! <https://262.ecma-international.org/14.0/#sec-object-initializer>

use crate::parser::{
    identifiers::parse_identifier_reference,
    tokenization::{Punctuator, SkipLineTerminators, Token, Tokenizer},
    SyntaxError,
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
