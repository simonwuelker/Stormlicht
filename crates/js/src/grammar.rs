use crate::{
    identifiers::BindingIdentifier, literals::Literal, tokenizer::Punctuator, SyntaxError,
    Tokenizer,
};

/// <https://262.ecma-international.org/14.0/#prod-ScriptBody>
#[derive(Clone, Debug)]
pub struct Script(Option<ScriptBody>);

impl Script {
    pub fn parse(tokenizer: &mut Tokenizer<'_>) -> Result<Self, SyntaxError> {
        let script_body = tokenizer.attempt(ScriptBody::parse).ok();
        Ok(Self(script_body))
    }
}

/// <https://262.ecma-international.org/14.0/#prod-ScriptBody>
#[derive(Clone, Debug)]
pub struct ScriptBody(StatementList);

impl ScriptBody {
    /// <https://262.ecma-international.org/14.0/#prod-ScriptBody>
    pub fn parse(tokenizer: &mut Tokenizer<'_>) -> Result<Self, SyntaxError> {
        let statement_list = StatementList::parse::<true, true, true>(tokenizer)?;
        Ok(Self(statement_list))
    }
}

/// <https://262.ecma-international.org/14.0/#prod-StatementList>
#[derive(Clone, Debug)]
pub struct StatementList(Vec<StatementListItem>);

impl StatementList {
    /// <https://262.ecma-international.org/14.0/#prod-StatementList>
    fn parse<const YIELD: bool, const AWAIT: bool, const RETURN: bool>(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Result<Self, SyntaxError> {
        let statements = vec![StatementListItem::parse::<YIELD, AWAIT, RETURN>(tokenizer)?];

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
pub struct Initializer(AssignmentExpression);

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

        let assignment_expression = AssignmentExpression::parse::<IN, YIELD, AWAIT>(tokenizer)?;

        Ok(Self(assignment_expression))
    }
}

/// <https://262.ecma-international.org/14.0/#prod-AssignmentExpression>
#[derive(Clone, Debug)]
pub enum AssignmentExpression {
    ConditionalExpression(ConditionalExpression),
    YIELDExpression,
    ArrowFunction,
    AsyncArrowFunction,
}

/// <https://262.ecma-international.org/14.0/#prod-ConditionalExpression>
// FIXME: incorrect
#[derive(Clone, Debug)]
pub struct ConditionalExpression(LeftHandSideExpression);

/// <https://262.ecma-international.org/14.0/#prod-LeftHandSideExpression>
#[derive(Clone, Debug)]
pub enum LeftHandSideExpression {
    NewExpression(NewExpression),
    CallExpression,
    OptionalExpression,
}

/// <https://262.ecma-international.org/14.0/#prod-NewExpression>
#[derive(Clone, Debug)]
pub struct NewExpression {
    /// The number of `new` keywords before the expression
    pub nest_level: usize,
    pub member_expression: MemberExpression,
}

/// <https://262.ecma-international.org/14.0/#prod-MemberExpression>
#[derive(Clone, Debug)]
pub enum MemberExpression {
    PrimaryExpression(PrimaryExpression),
    // FIXME: incomplete
}

/// <https://262.ecma-international.org/14.0/#prod-PrimaryExpression>
#[derive(Clone, Debug)]
pub enum PrimaryExpression {
    This,
    Literal(Literal),
    // FIXME: Incomplete
}

impl AssignmentExpression {
    fn parse<const IN: bool, const YIELD: bool, const AWAIT: bool>(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Result<Self, SyntaxError> {
        let conditional_expression = ConditionalExpression::parse::<IN, YIELD, AWAIT>(tokenizer)?;
        Ok(Self::ConditionalExpression(conditional_expression))
    }
}

impl ConditionalExpression {
    fn parse<const IN: bool, const YIELD: bool, const AWAIT: bool>(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Result<Self, SyntaxError> {
        let lhs_expression = LeftHandSideExpression::parse::<IN, YIELD, AWAIT>(tokenizer)?;
        Ok(Self(lhs_expression))
    }
}

impl LeftHandSideExpression {
    fn parse<const IN: bool, const YIELD: bool, const AWAIT: bool>(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Result<Self, SyntaxError> {
        let new_expression = NewExpression::parse::<IN, YIELD, AWAIT>(tokenizer)?;
        Ok(Self::NewExpression(new_expression))
    }
}

impl NewExpression {
    fn parse<const IN: bool, const YIELD: bool, const AWAIT: bool>(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Result<Self, SyntaxError> {
        let mut nest_level = 0;

        while matches!(
            tokenizer.attempt(Tokenizer::consume_identifier).as_deref(),
            Ok("new")
        ) {
            nest_level += 1;
        }

        let member_expression = MemberExpression::parse::<IN, YIELD, AWAIT>(tokenizer)?;
        Ok(Self {
            nest_level,
            member_expression,
        })
    }
}

impl MemberExpression {
    fn parse<const IN: bool, const YIELD: bool, const AWAIT: bool>(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Result<Self, SyntaxError> {
        let primary_expression = PrimaryExpression::parse::<IN, YIELD, AWAIT>(tokenizer)?;
        Ok(Self::PrimaryExpression(primary_expression))
    }
}

impl PrimaryExpression {
    fn parse<const IN: bool, const YIELD: bool, const AWAIT: bool>(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Result<Self, SyntaxError> {
        if matches!(
            tokenizer.attempt(Tokenizer::consume_identifier).as_deref(),
            Ok("this")
        ) {
            Ok(Self::This)
        } else if let Ok(literal) = Literal::parse(tokenizer) {
            Ok(Self::Literal(literal))
        } else {
            Err(tokenizer.syntax_error())
        }
    }
}
