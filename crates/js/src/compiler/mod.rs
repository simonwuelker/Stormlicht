//! Converts from the AST to bytecode

mod constant_store;
mod declarative_environment;

pub use constant_store::{ConstantHandle, ConstantStore};
pub use declarative_environment::Binding;

use std::{assert_matches::debug_assert_matches, rc::Rc};

use crate::{compiler::declarative_environment::DeclarativeEnvironment, parser, runtime, Value};

/// Compiler environment for a specific function
#[derive(Clone, Default)]
pub struct Compiler {
    bytecode: Vec<runtime::OpCode>,
    environment: Rc<DeclarativeEnvironment>,
    constants: ConstantStore,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JumpHandle(usize);

#[derive(Clone, Copy, Debug)]
pub enum Error {
    RedeclarationOfIdentifier,
    UndefinedIdentifierReference,
}

impl Compiler {
    /// Address used for jumps that need to be patched later
    const DUMMY_JUMP_ADDRESS: usize = usize::MAX;

    #[must_use]
    pub fn new_with_parent_environment(environment: Rc<DeclarativeEnvironment>) -> Self {
        Self {
            bytecode: Vec::default(),
            environment,
            constants: ConstantStore::default(),
        }
    }

    #[must_use]
    pub fn finish(self) -> runtime::Executable {
        runtime::Executable {
            num_variables: self.environment.len(),
            constants: self.constants,
            bytecode: self.bytecode,
        }
    }

    fn emit_opcode(&mut self, opcode: runtime::OpCode) {
        self.bytecode.push(opcode);
    }

    #[must_use]
    fn jump(&mut self) -> JumpHandle {
        let index = self.bytecode.len();
        self.emit_opcode(runtime::OpCode::Jump(Self::DUMMY_JUMP_ADDRESS));
        JumpHandle(index)
    }

    #[must_use]
    fn jump_if_false(&mut self) -> JumpHandle {
        let index = self.bytecode.len();
        self.emit_opcode(runtime::OpCode::JumpIfFalse(Self::DUMMY_JUMP_ADDRESS));
        JumpHandle(index)
    }

    /// Patch the given jump to the current position
    fn link_jump_handle(&mut self, handle: JumpHandle) {
        debug_assert_matches!(
            self.bytecode[handle.0],
            runtime::OpCode::Jump(Self::DUMMY_JUMP_ADDRESS),
            "Instruction is not an unpatched jump"
        );

        self.bytecode[handle.0] = runtime::OpCode::Jump(self.bytecode.len());
    }

    pub fn compile_script(&mut self, script: parser::Script) -> Result<(), Error> {
        self.compile_statement_list(script.statement_list())
    }

    fn compile_statement_list(
        &mut self,
        statement_list: &[parser::StatementListItem],
    ) -> Result<(), Error> {
        for statement_list_item in statement_list {
            self.compile_statement_list_item(statement_list_item)?;
        }

        Ok(())
    }

    fn compile_statement_list_item(
        &mut self,
        statement_list_item: &parser::StatementListItem,
    ) -> Result<(), Error> {
        match statement_list_item {
            parser::StatementListItem::Declaration(declaration) => {
                self.compile_declaration(declaration)
            },
            parser::StatementListItem::Statement(statement) => self.compile_statement(statement),
        }
    }

    fn compile_declaration(&mut self, declaration: &parser::Declaration) -> Result<(), Error> {
        match declaration {
            parser::Declaration::Function(function_declaration) => {
                self.compile_function_declaration(function_declaration)
            },
            parser::Declaration::Lexical(lexical_declaration) => {
                self.compile_lexical_declaration(lexical_declaration)
            },
        }
    }

    fn compile_function_declaration(
        &mut self,
        function_declaration: &parser::FunctionDeclaration,
    ) -> Result<(), Error> {
        let mut function_compiler = Self::new_with_parent_environment(self.environment.clone());
        function_compiler.compile_statement_list(function_declaration.body())?;
        todo!()
    }

    fn compile_lexical_declaration(
        &mut self,
        lexical_declaration: &parser::LexicalDeclaration,
    ) -> Result<(), Error> {
        for lexical_binding in lexical_declaration.lexical_bindings() {
            self.compile_lexical_binding(lexical_binding)?;
        }

        Ok(())
    }

    fn compile_lexical_binding(
        &mut self,
        lexical_binding: &parser::LexicalBinding,
    ) -> Result<(), Error> {
        match lexical_binding {
            parser::LexicalBinding::WithIdentifier {
                identifier,
                initializer,
            } => {
                let (binding, had_previous_binding) = self.environment.insert_binding(identifier);
                if had_previous_binding {
                    log::info!("Duplicate declaration of identifier {identifier:?}");
                    return Err(Error::RedeclarationOfIdentifier);
                }

                if let Some(initializer) = initializer {
                    self.compile_expression(initializer)?;
                    self.emit_opcode(runtime::OpCode::StoreTo(binding));
                }

                Ok(())
            },
        }
    }

    fn compile_statement(&mut self, statement: &parser::Statement) -> Result<(), Error> {
        match statement {
            parser::Statement::EmptyStatement => Ok(()),
            parser::Statement::BlockStatement(block) => self.compile_block_statement(block),
            parser::Statement::ExpressionStatement(expression) => {
                self.compile_expression(expression)
            },
            parser::Statement::IfStatement(if_statement) => self.compile_if_statement(if_statement),
            _ => todo!(),
        }
    }

    fn compile_if_statement(&mut self, if_statement: &parser::IfStatement) -> Result<(), Error> {
        self.compile_expression(&if_statement.condition())?;
        let false_case = self.jump_if_false();
        self.compile_statement(if_statement.if_branch())?;

        if let Some(else_branch) = if_statement.else_branch() {
            // Jump over else branch in true case
            let jump_handle = self.jump();

            self.link_jump_handle(false_case);
            self.compile_statement(else_branch)?;
            self.link_jump_handle(jump_handle);
        } else {
            self.link_jump_handle(false_case);
        }

        Ok(())
    }

    fn compile_block_statement(&mut self, block: &parser::BlockStatement) -> Result<(), Error> {
        self.compile_statement_list(block.statement_list())
    }

    /// Compile an Expression
    ///
    /// The result of the expression (which is always a single value) will
    /// be the topmost value on the stack afterwards
    fn compile_expression(&mut self, expression: &parser::Expression) -> Result<(), Error> {
        match expression {
            parser::Expression::This => todo!(),
            parser::Expression::Assignment(assignment_expression) => {
                self.compile_assignment_expression(assignment_expression)
            },
            parser::Expression::Binary(binary_expression) => {
                self.compile_binary_expression(binary_expression)
            },
            parser::Expression::Call(call_expression) => {
                self.compile_call_expression(call_expression)
            },
            parser::Expression::ConditionalExpression(conditional_expression) => {
                self.compile_conditional_expression(conditional_expression)
            },
            parser::Expression::Unary(unary_expression) => {
                self.compile_unary_expression(unary_expression)
            },
            parser::Expression::Update(update_expression) => {
                self.compile_update_expression(update_expression)
            },
            parser::Expression::IdentifierReference(identifier_reference) => {
                self.compile_identifier_reference(identifier_reference)
            },
            parser::Expression::Literal(literal) => {
                self.compile_literal(literal);
                Ok(())
            },
            parser::Expression::ObjectLiteral(_) => todo!(),
            parser::Expression::New(new_expression) => self.compile_new_expression(new_expression),
            parser::Expression::Member(member_expression) => {
                self.compile_member_expression(member_expression)
            },
        }
    }

    fn compile_literal(&mut self, literal: &parser::Literal) {
        let constant = Value::from(literal.to_owned());
        let handle = self.constants.get_or_insert_constant(constant);
        self.emit_opcode(runtime::OpCode::LoadConstant(handle));
    }

    fn compile_identifier_reference(&mut self, identifier: &str) -> Result<(), Error> {
        let Some(binding) = self.environment.locate_binding(identifier) else {
            // Undefined identifier reference
            log::error!("Undefined reference to identifier {identifier:?}");
            return Err(Error::UndefinedIdentifierReference);
        };

        self.emit_opcode(runtime::OpCode::LoadFrom(binding));
        Ok(())
    }

    fn compile_assignment_expression(
        &mut self,
        assignment_expression: &parser::AssignmentExpression,
    ) -> Result<(), Error> {
        _ = assignment_expression;
        todo!();
    }

    fn compile_binary_expression(
        &mut self,
        binary_expression: &parser::BinaryExpression,
    ) -> Result<(), Error> {
        self.compile_expression(binary_expression.right_hand_side())?;
        self.compile_expression(binary_expression.left_hand_side())?;

        self.emit_opcode(binary_expression.operator().into());
        Ok(())
    }

    fn compile_call_expression(
        &mut self,
        call_expression: &parser::CallExpression,
    ) -> Result<(), Error> {
        _ = call_expression;
        todo!()
    }

    fn compile_conditional_expression(
        &mut self,
        conditional_expression: &parser::ConditionalExpression,
    ) -> Result<(), Error> {
        _ = conditional_expression;
        todo!()
    }

    fn compile_unary_expression(
        &mut self,
        unary_expression: &parser::UnaryExpression,
    ) -> Result<(), Error> {
        _ = unary_expression;
        todo!()
    }

    fn compile_update_expression(
        &mut self,
        update_expression: &parser::UpdateExpression,
    ) -> Result<(), Error> {
        _ = update_expression;
        todo!()
    }

    fn compile_member_expression(
        &mut self,
        member_expression: &parser::MemberExpression,
    ) -> Result<(), Error> {
        _ = member_expression;
        todo!()
    }

    fn compile_new_expression(
        &mut self,
        new_expression: &parser::NewExpression,
    ) -> Result<(), Error> {
        _ = new_expression;
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use super::*;

    #[test]
    fn undefined_identifier_should_not_compile() {
        let program: Result<runtime::Executable, Error> = "let y = x + 1;".parse();

        assert_matches!(program, Err(Error::UndefinedIdentifierReference));
    }
}
