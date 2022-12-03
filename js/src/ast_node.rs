use crate::interpreter::JSInterpreter;

pub enum BinaryOperator {
    Add,
}

pub enum VariableDeclarationKind {
    Let,
}

pub trait ASTNode {
    fn traverse(&self, interpreter: &mut JSInterpreter);
}

pub type ASTElement = Box<dyn ASTNode>;

pub struct Identifier {
    name: String,
}

pub struct VariableDeclarations {
    kind: VariableDeclarationKind,
    declarations: Vec<ASTElement>,
}

pub struct VariableDeclarator {
    id: ASTElement,
    init: ASTElement,
}

pub struct BinaryExpression {
    operator: BinaryOperator,
    left: ASTElement,
    right: ASTElement,
}

impl ASTNode for Identifier {
    fn traverse(&self, _interpreter: &mut JSInterpreter) {
        todo!();
    }
}
