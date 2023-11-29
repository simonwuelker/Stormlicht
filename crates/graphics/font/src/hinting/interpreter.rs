use super::op;

const MAX_STORAGE_AREAS_TO_RESERVE: usize = 32;
const MAX_FUNCTION_DEFS_TO_RESERVE: usize = 32;

#[derive(Clone, Copy, Debug)]
pub enum Error {
    UnknownInstruction(u8),

    /// Encountered EOF in the middle of an instruction
    EndOfFileInInstruction,

    /// A `FDEF` with no corresponding `ENDF`
    UnterminatedFunctionDefinition,

    /// Need a value to be popped from the stack, but its empty
    EmptyStack,

    /// Tried to call a function that was not defined
    UndefinedFunction,

    /// A `ENDF` with no corresponding `FDEF`
    UnexpectedEndf,

    /// Trying to access more storage than was requested from the `maxp` table
    StorageAddressOutOfRange,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum IterationDecision {
    Break,
    Continue,
}

#[derive(Clone, Debug)]
pub struct Interpreter {
    storage_areas: Box<[u8]>,
    stack: Vec<u32>,
    function_definitions: Box<[Option<Vec<u8>>]>,
}

impl Interpreter {
    pub fn new(max_storage_areas: usize, max_function_defs: usize) -> Self {
        let storage_areas = if MAX_STORAGE_AREAS_TO_RESERVE < max_storage_areas {
            log::error!("Font needs {max_storage_areas} storage areas, but we won't reserve more than {MAX_STORAGE_AREAS_TO_RESERVE}");
            log::error!("This means that executing font programs will likely fail!");

            vec![0; MAX_STORAGE_AREAS_TO_RESERVE].into_boxed_slice()
        } else {
            vec![0; max_storage_areas].into_boxed_slice()
        };

        let function_definitions = if MAX_FUNCTION_DEFS_TO_RESERVE < max_function_defs {
            log::error!("Font needs {max_function_defs} function definitions, but we won't reserve more than {MAX_FUNCTION_DEFS_TO_RESERVE}");
            log::error!("This means that executing font programs will likely fail!");

            vec![None; MAX_FUNCTION_DEFS_TO_RESERVE].into_boxed_slice()
        } else {
            vec![None; max_function_defs].into_boxed_slice()
        };

        Self {
            storage_areas,
            stack: Vec::new(),
            function_definitions,
        }
    }

    pub fn run(&mut self, instruction_stream: &[u8]) -> Result<(), Error> {
        let mut program = ExecutionContext::new(instruction_stream);
        loop {
            let result = self.execute_instruction(&mut program)?;

            if result == IterationDecision::Break {
                break;
            }
        }
        Ok(())
    }

    fn execute_instruction(
        &mut self,
        program: &mut ExecutionContext<'_>,
    ) -> Result<IterationDecision, Error> {
        match program.next_u8() {
            Some(op::CALL) => {
                // Call a previously defined function
                let function_identifier = self.stack.pop().ok_or(Error::EmptyStack)?;

                let function = self
                    .function_definitions
                    .get(function_identifier as usize)
                    .ok_or(Error::UndefinedFunction)?;

                match function {
                    Some(instructions) => {
                        // FIXME: This clone is a little ugly
                        self.run(&instructions.clone())?;
                    },
                    None => {
                        return Err(Error::UndefinedFunction);
                    },
                }
            },
            Some(op::FDEF) => {
                // Function definition
                let function_identifier = self.stack.pop().ok_or(Error::EmptyStack)?;
                let function_body = program.consume_function_definition()?;
                self.function_definitions[function_identifier as usize] =
                    Some(function_body.to_owned());
            },
            Some(op::ENDF) => {
                // ENDF (End Function)
                // (only legal if we are within a function definition)
                return Err(Error::UnexpectedEndf);
            },
            Some(op::RS) => {
                // RS (Read Storage)
                let address = self.stack.pop().ok_or(Error::EmptyStack)?;
                let storage_value = self
                    .storage_areas
                    .get(address as usize)
                    .ok_or(Error::StorageAddressOutOfRange)?;
                self.stack.push(*storage_value as u32);
            },
            Some(op::MPPEM) => {
                // MPPEM (Measure Pixels Per Em)
            },
            Some(n @ (op::PUSHB_START..=op::PUSHB_END)) => {
                // Push bytes on stack
                let n_bytes_to_push = n - op::PUSHB_START + 1;

                for _ in 0..n_bytes_to_push {
                    let b = program.next_u8().ok_or(Error::EndOfFileInInstruction)?;
                    self.stack.push(b as u32);
                }
            },
            Some(other) => return Err(Error::UnknownInstruction(other)),
            None => return Ok(IterationDecision::Break),
        }
        Ok(IterationDecision::Continue)
    }
}

#[derive(Clone, Copy)]
struct ExecutionContext<'a> {
    bytes: &'a [u8],
    cursor: usize,
}

impl<'a> ExecutionContext<'a> {
    #[must_use]
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, cursor: 0 }
    }

    #[must_use]
    fn remaining(&self) -> &[u8] {
        &self.bytes[self.cursor..]
    }

    fn consume_function_definition(&mut self) -> Result<&[u8], Error> {
        let end_index = self
            .remaining()
            .iter()
            .position(|&op| op == op::ENDF)
            .ok_or(Error::UnterminatedFunctionDefinition)?;
        let function_body = &self.remaining()[..end_index];
        Ok(function_body)
    }

    fn next_u8(&mut self) -> Option<u8> {
        let byte = self.bytes.get(self.cursor).copied();
        self.cursor += 1;
        byte
    }
}
