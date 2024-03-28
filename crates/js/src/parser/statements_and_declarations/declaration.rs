//! <https://262.ecma-international.org/14.0/#sec-declarations-and-the-variable-statement>

use crate::{
    bytecode::{self, CompileToBytecode},
    parser::{
        expressions::Expression,
        functions_and_classes::FunctionDeclaration,
        identifiers::parse_binding_identifier,
        tokenization::{Punctuator, SkipLineTerminators, Token, Tokenizer},
        SyntaxError,
    },
};

/// <https://262.ecma-international.org/14.0/#prod-Declaration>
#[derive(Clone, Debug)]
pub enum Declaration {
    Function(FunctionDeclaration),
    Lexical(LexicalDeclaration),
}

impl Declaration {
    /// <https://262.ecma-international.org/14.0/#prod-Declaration>
    pub fn parse<const YIELD: bool, const AWAIT: bool>(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Result<Self, SyntaxError> {
        let Some(next_token) = tokenizer.peek(0, SkipLineTerminators::Yes)? else {
            return Err(tokenizer.syntax_error());
        };

        let declaration = match next_token {
            Token::Identifier(ident) if ident == "function" => {
                FunctionDeclaration::parse::<YIELD, AWAIT, true>(tokenizer)?.into()
            },
            Token::Identifier(ident) if matches!(ident.as_str(), "let" | "const") => {
                LexicalDeclaration::parse::<true, YIELD, AWAIT>(tokenizer)?.into()
            },
            _ => return Err(tokenizer.syntax_error()),
        };

        Ok(declaration)
    }
}

/// <https://262.ecma-international.org/14.0/#prod-LetOrConst>
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LetOrConst {
    Let,
    Const,
}

/// <https://262.ecma-international.org/14.0/#prod-LexicalDeclaration>
#[derive(Clone, Debug)]
pub struct LexicalDeclaration {
    pub let_or_const: LetOrConst,
    pub lexical_bindings: Vec<LexicalBinding>,
}

impl LetOrConst {
    /// <https://262.ecma-international.org/14.0/#prod-LetOrConst>
    fn parse(tokenizer: &mut Tokenizer<'_>) -> Result<Self, SyntaxError> {
        let let_or_const = match tokenizer.next(SkipLineTerminators::Yes)? {
            Some(Token::Identifier(ident)) if ident == "let" => Self::Let,
            Some(Token::Identifier(ident)) if ident == "const" => Self::Const,
            _ => return Err(tokenizer.syntax_error()),
        };

        Ok(let_or_const)
    }
}

impl LexicalDeclaration {
    /// <https://262.ecma-international.org/14.0/#prod-LexicalDeclaration>
    fn parse<const IN: bool, const YIELD: bool, const AWAIT: bool>(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Result<Self, SyntaxError> {
        let let_or_const = LetOrConst::parse(tokenizer)?;
        let first_lexical_binding = LexicalBinding::parse::<IN, YIELD, AWAIT>(tokenizer)?;
        let mut lexical_bindings = vec![first_lexical_binding];

        while matches!(
            tokenizer.peek(0, SkipLineTerminators::Yes)?,
            Some(Token::Punctuator(Punctuator::Comma))
        ) {
            tokenizer.advance(1);

            let lexical_binding = LexicalBinding::parse::<IN, YIELD, AWAIT>(tokenizer)?;
            lexical_bindings.push(lexical_binding);
        }

        let lexical_declaration = Self {
            let_or_const,
            lexical_bindings,
        };

        tokenizer.expect_semicolon()?;

        Ok(lexical_declaration)
    }
}

/// <https://262.ecma-international.org/14.0/#prod-LexicalBinding>
#[derive(Clone, Debug)]
pub enum LexicalBinding {
    WithIdentifier {
        identifier: String,
        initializer: Option<Expression>,
    },
    BindingPattern,
}

impl LexicalBinding {
    /// <https://262.ecma-international.org/14.0/#prod-LexicalBinding>
    fn parse<const IN: bool, const YIELD: bool, const AWAIT: bool>(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Result<Self, SyntaxError> {
        let Some(next_token) = tokenizer.peek(0, SkipLineTerminators::Yes)? else {
            return Err(tokenizer.syntax_error());
        };

        let lexical_binding = match next_token {
            Token::Identifier(_) => {
                let identifier = parse_binding_identifier::<YIELD, AWAIT>(tokenizer)?;

                let has_initializer = matches!(
                    tokenizer.peek(0, SkipLineTerminators::Yes)?,
                    Some(Token::Punctuator(Punctuator::Equal))
                );
                let initializer = if has_initializer {
                    Some(parse_initializer::<IN, YIELD, AWAIT>(tokenizer)?)
                } else {
                    None
                };

                Self::WithIdentifier {
                    identifier,
                    initializer,
                }
            },
            Token::Punctuator(Punctuator::BracketOpen) => {
                log::error!("Unimplemented: ArrayBindingPattern in LexicalBinding");
                return Err(tokenizer.syntax_error());
            },
            Token::Punctuator(Punctuator::CurlyBraceOpen) => {
                log::error!("Unimplemented: ObjectBindingPattern in LexicalBinding");
                return Err(tokenizer.syntax_error());
            },
            _ => return Err(tokenizer.syntax_error()),
        };

        Ok(lexical_binding)
    }
}

/// <https://262.ecma-international.org/14.0/#prod-Initializer>
fn parse_initializer<const IN: bool, const YIELD: bool, const AWAIT: bool>(
    tokenizer: &mut Tokenizer<'_>,
) -> Result<Expression, SyntaxError> {
    tokenizer.expect_punctuator(Punctuator::Equal)?;

    // FIXME: This should be an AssignmentExpression, not an Expression
    let assignment_expression = Expression::parse::<IN, YIELD, AWAIT>(tokenizer)?;

    Ok(assignment_expression)
}

impl CompileToBytecode for Declaration {
    fn compile(&self, builder: &mut bytecode::ProgramBuilder) {
        match self {
            Self::Lexical(lexical_declaration) => lexical_declaration.compile(builder),
            Self::Function(function_declaration) => function_declaration.compile(builder),
        }
    }
}

impl CompileToBytecode for LexicalDeclaration {
    fn compile(&self, builder: &mut bytecode::ProgramBuilder) -> Self::Result {
        let current_block = builder.current_block();

        for lexical_binding in &self.lexical_bindings {
            match lexical_binding {
                LexicalBinding::WithIdentifier {
                    identifier,
                    initializer,
                } => {
                    builder
                        .get_block(current_block)
                        .create_variable(&identifier);

                    if let Some(expression) = initializer {
                        let result = expression.compile(builder);
                        builder
                            .get_block(current_block)
                            .update_variable(identifier.clone(), result);
                    }
                },
                LexicalBinding::BindingPattern => todo!(),
            }
        }
    }
}

impl From<FunctionDeclaration> for Declaration {
    fn from(value: FunctionDeclaration) -> Self {
        Self::Function(value)
    }
}

impl From<LexicalDeclaration> for Declaration {
    fn from(value: LexicalDeclaration) -> Self {
        Self::Lexical(value)
    }
}
