mod builder;
mod instruction;
mod vm;

pub use builder::{Builder, Register};
pub use instruction::{Instruction, VariableHandle};
pub use vm::{Value, Vm};

pub trait CompileToBytecode {
    type Result = ();

    fn compile(&self, builder: &mut Builder) -> Self::Result;
}
