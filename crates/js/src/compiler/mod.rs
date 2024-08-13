//! Converts from the AST to bytecode

mod declarative_environment;

use std::{assert_matches::debug_assert_matches, rc::Rc};

use crate::{compiler::declarative_environment::DeclarativeEnvironment, parser, runtime, Value};

/// Compiler environment for a specific function
#[derive(Clone, Default)]
pub struct Compiler {
    bytecode: Vec<runtime::OpCode>,
    environment: Rc<DeclarativeEnvironment>,
    constants: Vec<Value>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConstantHandle(usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JumpHandle(usize);

impl Compiler {
    /// Address used for jumps that need to be patched later
    const DUMMY_JUMP_ADDRESS: usize = usize::MAX;

    #[must_use]
    pub fn new_with_parent_environment(environment: Rc<DeclarativeEnvironment>) -> Self {
        Self {
            bytecode: Vec::default(),
            environment,
            constants: Vec::default(),
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

    #[must_use]
    pub fn get_or_insert_constant(&mut self, constant: Value) -> ConstantHandle {
        let index = self
            .constants
            .iter()
            .position(|v| v == &constant)
            .unwrap_or_else(|| {
                let index = self.constants.len();
                self.constants.push(constant);
                index
            });

        ConstantHandle(index)
    }

    pub fn compile_script(&mut self, script: parser::Script) {
        self.compile_statement_list(script.statement_list())
    }

    fn compile_statement_list(&mut self, statement_list: &[parser::StatementListItem]) {
        for statement_list_item in statement_list {
            self.compile_statement_list_item(statement_list_item);
        }
    }

    fn compile_statement_list_item(&mut self, statement_list_item: &parser::StatementListItem) {
        match statement_list_item {
            parser::StatementListItem::Declaration(declaration) => {
                self.compile_declaration(declaration)
            },
            parser::StatementListItem::Statement(statement) => self.compile_statement(statement),
        }
    }

    fn compile_declaration(&mut self, declaration: &parser::Declaration) {
        match declaration {
            parser::Declaration::Function(function_declaration) => {
                self.compile_function_declaration(function_declaration)
            },
            parser::Declaration::Lexical(lexical_declaration) => {
                self.compile_lexical_declaration(lexical_declaration)
            },
        }
    }

    fn compile_function_declaration(&mut self, function_declaration: &parser::FunctionDeclaration) {
        let mut function_compiler = Self::new_with_parent_environment(self.environment.clone());
        function_compiler.compile_statement_list(function_declaration.body());
        todo!()
    }

    fn compile_lexical_declaration(&mut self, lexical_declaration: &parser::LexicalDeclaration) {
        for lexical_binding in lexical_declaration.lexical_bindings() {
            match lexical_binding {
                parser::LexicalBinding::WithIdentifier { identifier, .. } => {
                    self.environment.insert_binding(identifier);
                },
            }
        }
    }

    fn compile_statement(&mut self, statement: &parser::Statement) {
        match statement {
            parser::Statement::EmptyStatement => {},
            parser::Statement::BlockStatement(block) => self.compile_block_statement(block),
            parser::Statement::ExpressionStatement(expression) => {
                self.compile_expression(expression)
            },
            parser::Statement::IfStatement(if_statement) => self.compile_if_statement(if_statement),
            _ => todo!(),
        }
    }

    fn compile_if_statement(&mut self, if_statement: &parser::IfStatement) {
        self.compile_expression(&if_statement.condition());
        let false_case = self.jump_if_false();
        self.compile_statement(if_statement.if_branch());

        if let Some(else_branch) = if_statement.else_branch() {
            // Jump over else branch in true case
            let jump_handle = self.jump();

            self.link_jump_handle(false_case);
            self.compile_statement(else_branch);
            self.link_jump_handle(jump_handle);
        } else {
            self.link_jump_handle(false_case);
        }
    }

    fn compile_block_statement(&mut self, block: &parser::BlockStatement) {
        self.compile_statement_list(block.statement_list());
    }

    /// Compile an Expression
    ///
    /// The result of the expression (which is always a single value) will
    /// be the topmost value on the stack afterwards
    fn compile_expression(&mut self, expression: &parser::Expression) {
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
                self.compile_identifier_reference(identifier_reference);
            },
            parser::Expression::Literal(literal) => self.compile_literal(literal),
            parser::Expression::ObjectLiteral(_) => todo!(),
            parser::Expression::New(new_expression) => self.compile_new_expression(new_expression),
            parser::Expression::Member(member_expression) => {
                self.compile_member_expression(member_expression)
            },
        }
    }

    fn compile_literal(&mut self, literal: &parser::Literal) {
        let constant = Value::from(literal.to_owned());
        let handle = self.get_or_insert_constant(constant);
    }

    fn compile_identifier_reference(&mut self, identifier: &str) {
        // let binding = self.environment.get()
    }

    fn compile_assignment_expression(
        &mut self,
        assignment_expression: &parser::AssignmentExpression,
    ) {
        _ = assignment_expression;
        todo!();
    }

    fn compile_binary_expression(&mut self, binary_expression: &parser::BinaryExpression) {
        self.compile_expression(binary_expression.right_hand_side());
        self.compile_expression(binary_expression.left_hand_side());

        self.bytecode.push(binary_expression.operator().into());
    }

    fn compile_call_expression(&mut self, call_expression: &parser::CallExpression) {
        _ = call_expression;
        todo!()
    }

    fn compile_conditional_expression(
        &mut self,
        conditional_expression: &parser::ConditionalExpression,
    ) {
        _ = conditional_expression;
        todo!()
    }

    fn compile_unary_expression(&mut self, unary_expression: &parser::UnaryExpression) {
        _ = unary_expression;
        todo!()
    }

    fn compile_update_expression(&mut self, update_expression: &parser::UpdateExpression) {
        _ = update_expression;
        todo!()
    }

    fn compile_member_expression(&mut self, member_expression: &parser::MemberExpression) {
        _ = member_expression;
        todo!()
    }

    fn compile_new_expression(&mut self, new_expression: &parser::NewExpression) {
        _ = new_expression;
        todo!()
    }
}
