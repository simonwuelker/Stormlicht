mod builder;
mod exception;
mod instruction;
mod value;
mod vm;

pub use builder::{Builder, Register};
pub use exception::{Exception, ThrowCompletionOr};
pub use instruction::{Instruction, VariableHandle};
pub use value::Value;
pub use vm::Vm;

pub trait CompileToBytecode {
    type Result = ();

    fn compile(&self, builder: &mut Builder) -> Self::Result;
}
