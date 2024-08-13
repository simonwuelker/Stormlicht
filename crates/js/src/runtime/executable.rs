use std::str::FromStr;

use crate::{compiler, parser, Value};

use super::OpCode;

/// Represents a compiled ecmascript program
#[derive(Clone, Debug)]
pub struct Executable {
    pub num_variables: usize,
    pub constants: compiler::ConstantStore,
    pub bytecode: Vec<OpCode>,
}

impl Executable {
    #[must_use]
    pub fn fetch_instruction(&self, program_counter: usize) -> Option<OpCode> {
        self.bytecode.get(program_counter).copied()
    }

    #[must_use]
    pub fn fetch_constant(&self, handle: compiler::ConstantHandle) -> &Value {
        self.constants.get_constant(handle)
    }
}

impl FromStr for Executable {
    type Err = compiler::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use parser::{GoalSymbol, Script, Tokenizer};

        let mut tokenizer = Tokenizer::new(s, GoalSymbol::Script);
        let script = Script::parse(&mut tokenizer).unwrap(); // FIXME
        let mut compiler = compiler::Compiler::default();
        compiler.compile_script(script)?;
        let executable = compiler.finish();

        Ok(executable)
    }
}
