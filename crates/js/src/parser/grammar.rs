use super::{
    identifiers::BindingIdentifier, tokenizer::Punctuator, Expression, SyntaxError, Tokenizer,
};

/// <https://262.ecma-international.org/14.0/#prod-ScriptBody>
#[derive(Clone, Debug)]
pub struct Script(Vec<StatementListItem>);

impl Script {
    pub fn parse(tokenizer: &mut Tokenizer<'_>) -> Result<Self, SyntaxError> {
        let statements = vec![StatementListItem::parse::<true, true, true>(tokenizer)?];

        // FIXME: parse more than one statement here
        Ok(Self(statements))
    }
}

/// <https://262.ecma-international.org/14.0/#prod-StatementListItem>
#[derive(Clone, Debug)]
enum StatementListItem {
    Statement(Statement),
    Declaration(Declaration),
}

impl StatementListItem {
    /// <https://262.ecma-international.org/14.0/#prod-StatementListItem>
    fn parse<const YIELD: bool, const AWAIT: bool, const RETURN: bool>(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Result<Self, SyntaxError> {
        let statement_list_item =
            if let Ok(statement) = tokenizer.attempt(Statement::parse::<YIELD, AWAIT, RETURN>) {
                Self::Statement(statement)
            } else if let Ok(declaration) = tokenizer.attempt(Declaration::parse::<YIELD, AWAIT>) {
                Self::Declaration(declaration)
            } else {
                return Err(tokenizer.syntax_error());
            };

        Ok(statement_list_item)
    }
}

/// <https://262.ecma-international.org/14.0/#prod-Statement>
#[derive(Clone, Debug)]
pub enum Statement {
    BlockStatement,
    VariableStatement,
    EmptyStatement,
    ExpressionStatement,
    IfStatement,
    BreakableStatement,
    ContinueStatement,
    BreakStatement,
    RETURNStatement,
    WithStatement,
    LabelledStatement,
    ThrowStatement,
    TryStatement,
    DebuggerStatement,
}

impl Statement {
    /// <https://262.ecma-international.org/14.0/#prod-Statement>
    fn parse<const YIELD: bool, const AWAIT: bool, const RETURN: bool>(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Result<Self, SyntaxError> {
        // TODO
        _ = tokenizer;
        Err(tokenizer.syntax_error())
    }
}

/// <https://262.ecma-international.org/14.0/#prod-Declaration>
#[derive(Clone, Debug)]
pub enum Declaration {
    HoistableDeclaration,
    ClassDeclaration,
    LexicalDeclaration(LexicalDeclaration),
}

impl Declaration {
    /// <https://262.ecma-international.org/14.0/#prod-Declaration>
    fn parse<const YIELD: bool, const AWAIT: bool>(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Result<Self, SyntaxError> {
        let declaration = if let Ok(lexical_declaration) =
            tokenizer.attempt(LexicalDeclaration::parse::<true, YIELD, AWAIT>)
        {
            Self::LexicalDeclaration(lexical_declaration)
        } else {
            return Err(tokenizer.syntax_error());
        };

        Ok(declaration)
    }
}

/// <https://262.ecma-international.org/14.0/#prod-LexicalDeclaration>
#[derive(Clone, Debug)]
pub struct LexicalDeclaration {
    pub let_or_const: LetOrConst,
    pub binding_list: Vec<LexicalBinding>,
}

/// <https://262.ecma-international.org/14.0/#prod-LetOrConst>
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LetOrConst {
    Let,
    Const,
}

/// <https://262.ecma-international.org/14.0/#prod-LexicalBinding>
#[derive(Clone, Debug)]
pub enum LexicalBinding {
    WithIdentifier {
        identifier: BindingIdentifier,
        initializer: Option<Initializer>,
    },
    BindingPattern,
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

impl LexicalBinding {
    /// <https://262.ecma-international.org/14.0/#prod-LexicalBinding>
    fn parse<const IN: bool, const YIELD: bool, const AWAIT: bool>(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Result<Self, SyntaxError> {
        let with_identifier = |tokenizer: &mut Tokenizer<'_>| {
            let identifier = tokenizer.attempt(BindingIdentifier::parse::<IN, YIELD>)?;
            let initializer = tokenizer
                .attempt(Initializer::parse::<IN, YIELD, AWAIT>)
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
#[derive(Clone, Debug)]
pub struct Initializer(Expression);

impl Initializer {
    fn parse<const IN: bool, const YIELD: bool, const AWAIT: bool>(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Result<Self, SyntaxError> {
        if !matches!(
            tokenizer.attempt(Tokenizer::consume_punctuator)?,
            Punctuator::Equal
        ) {
            return Err(tokenizer.syntax_error());
        }

        // FIXME: This should be an AssignmentExpression, not an Expression
        let assignment_expression = Expression::parse::<IN, YIELD, AWAIT>(tokenizer)?;

        Ok(Self(assignment_expression))
    }
}
