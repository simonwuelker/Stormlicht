//! <https://262.ecma-international.org/14.0/#sec-if-statement>

use crate::{
    bytecode::{self, CompileToBytecode},
    parser::{expressions::Expression, tokenizer::Punctuator, SyntaxError, Tokenizer},
};

use super::Statement;

/// <https://262.ecma-international.org/14.0/#prod-IfStatement>
#[derive(Clone, Debug)]
pub struct IfStatement {
    pub condition: Expression,
    pub if_branch: Box<Statement>,
    pub else_branch: Option<Box<Statement>>,
}

impl IfStatement {
    /// <https://262.ecma-international.org/14.0/#prod-IfStatement>
    pub fn parse<const YIELD: bool, const AWAIT: bool, const RETURN: bool>(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Result<Self, SyntaxError> {
        tokenizer.expect_keyword("if")?;
        tokenizer.expect_punctuator(Punctuator::ParenthesisOpen)?;
        let condition = Expression::parse::<true, YIELD, AWAIT>(tokenizer)?;
        tokenizer.expect_punctuator(Punctuator::ParenthesisClose)?;

        let if_branch = Statement::parse::<YIELD, AWAIT, RETURN>(tokenizer)?;

        let else_branch = tokenizer
            .attempt(parse_else_branch::<YIELD, AWAIT, RETURN>)
            .ok();

        let if_statement = Self {
            condition,
            if_branch: Box::new(if_branch),
            else_branch: else_branch.map(Box::new),
        };

        Ok(if_statement)
    }
}

fn parse_else_branch<const YIELD: bool, const AWAIT: bool, const RETURN: bool>(
    tokenizer: &mut Tokenizer<'_>,
) -> Result<Statement, SyntaxError> {
    tokenizer.expect_keyword("else")?;
    let else_branch = Statement::parse::<YIELD, AWAIT, RETURN>(tokenizer)?;

    Ok(else_branch)
}

impl CompileToBytecode for IfStatement {
    fn compile(&self, builder: &mut bytecode::ProgramBuilder) {
        let condition_register = self.condition.compile(builder);
        let branching_block = builder.current_block();

        // The block that the two different executions will join on again
        let after_block = builder.allocate_basic_block();

        // Compile the "if" branch
        let if_block = builder.allocate_basic_block();
        builder.set_current_block(if_block);
        self.if_branch.compile(builder);
        builder
            .get_current_block()
            .unconditionally_jump_to(after_block);

        if let Some(else_branch) = self.else_branch.as_ref() {
            // Compile the "else" branch
            let else_block = builder.allocate_basic_block();
            builder.set_current_block(else_block);
            else_branch.compile(builder);
            builder
                .get_current_block()
                .unconditionally_jump_to(after_block);

            // Branch to either the "if" or the "else" branch
            builder
                .get_block(branching_block)
                .branch_if(condition_register, if_block, else_block);
        } else {
            // Branch to either the "if" or the "after" branch
            builder
                .get_block(branching_block)
                .branch_if(condition_register, if_block, after_block);
        }

        builder.set_current_block(after_block);
    }
}
