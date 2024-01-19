use crate::bytecode::{self, CompileToBytecode};

use super::{identifiers, tokenizer::Punctuator, Expression, SyntaxError, Tokenizer};

/// <https://262.ecma-international.org/14.0/#prod-Declaration>
#[derive(Clone, Debug)]
pub enum Declaration {
    Lexical(LexicalDeclaration),
}

impl Declaration {
    /// <https://262.ecma-international.org/14.0/#prod-Declaration>
    pub fn parse<const YIELD: bool, const AWAIT: bool>(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Result<Self, SyntaxError> {
        let declaration = if let Ok(lexical_declaration) =
            tokenizer.attempt(LexicalDeclaration::parse::<true, YIELD, AWAIT>)
        {
            Self::Lexical(lexical_declaration)
        } else {
            return Err(tokenizer.syntax_error());
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
    pub binding_list: Vec<LexicalBinding>,
}

impl LetOrConst {
    /// <https://262.ecma-international.org/14.0/#prod-LetOrConst>
    fn parse(tokenizer: &mut Tokenizer<'_>) -> Result<Self, SyntaxError> {
        let let_or_const = match tokenizer.attempt(Tokenizer::consume_identifier)?.as_str() {
            "let" => Self::Let,
            "const" => Self::Const,
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
        let lexical_binding = LexicalBinding::parse::<IN, YIELD, AWAIT>(tokenizer)?;

        let lexical_declaration = Self {
            let_or_const,
            binding_list: vec![lexical_binding],
        };

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
        let with_identifier = |tokenizer: &mut Tokenizer<'_>| {
            let identifier =
                tokenizer.attempt(identifiers::parse_binding_identifier::<IN, YIELD>)?;
            let initializer = tokenizer
                .attempt(parse_initializer::<IN, YIELD, AWAIT>)
                .ok();
            Ok(Self::WithIdentifier {
                identifier,
                initializer,
            })
        };

        tokenizer.attempt(with_identifier)
    }
}

/// <https://262.ecma-international.org/14.0/#prod-Initializer>
fn parse_initializer<const IN: bool, const YIELD: bool, const AWAIT: bool>(
    tokenizer: &mut Tokenizer<'_>,
) -> Result<Expression, SyntaxError> {
    if !matches!(
        tokenizer.attempt(Tokenizer::consume_punctuator)?,
        Punctuator::Equal
    ) {
        return Err(tokenizer.syntax_error());
    }

    // FIXME: This should be an AssignmentExpression, not an Expression
    let assignment_expression = Expression::parse::<IN, YIELD, AWAIT>(tokenizer)?;

    Ok(assignment_expression)
}

impl CompileToBytecode for Declaration {
    fn compile(&self, builder: &mut bytecode::Builder) {
        match self {
            Self::Lexical(lexical_declaration) => lexical_declaration.compile(builder),
        }
    }
}

impl CompileToBytecode for LexicalDeclaration {
    fn compile(&self, builder: &mut bytecode::Builder) -> Self::Result {
        for lexical_binding in &self.binding_list {
            match lexical_binding {
                LexicalBinding::WithIdentifier {
                    identifier,
                    initializer,
                } => {
                    let handle = builder.create_variable(&identifier);

                    if let Some(expression) = initializer {
                        let result = expression.compile(builder);
                        builder.update_variable(handle, result);
                    }
                },
                LexicalBinding::BindingPattern => todo!(),
            }
        }
    }
}
