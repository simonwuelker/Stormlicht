//! <https://262.ecma-international.org/14.0/#sec-throw-statement>

use crate::{
    bytecode::{self, CompileToBytecode},
    parser::{expressions::Expression, tokenization::Tokenizer, SyntaxError},
};

/// <https://262.ecma-international.org/14.0/#sec-throw-statement>
#[derive(Clone, Debug)]
pub struct ThrowStatement {
    expression: Expression,
}

impl ThrowStatement {
    /// <https://262.ecma-international.org/14.0/#prod-ThrowStatement>
    pub fn parse<const YIELD: bool, const AWAIT: bool>(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Result<Self, SyntaxError> {
        tokenizer.expect_keyword("throw")?;

        tokenizer.expect_no_line_terminator()?;
        let expression = Expression::parse::<true, YIELD, AWAIT>(tokenizer)?;

        let throw_statement = Self { expression };

        Ok(throw_statement)
    }
}

impl CompileToBytecode for ThrowStatement {
    fn compile(&self, builder: &mut bytecode::ProgramBuilder) -> Self::Result {
        // <https://262.ecma-international.org/14.0/#sec-throw-statement-runtime-semantics-evaluation>

        // 1. Let exprRef be ? Evaluation of Expression.
        let expr_ref = self.expression.compile(builder);

        // FIXME: 2. Let exprValue be ? GetValue(exprRef).
        let expr_value = expr_ref;

        // 3. Return ThrowCompletion(exprValue).
        builder.get_current_block().throw(expr_value);
    }
}
