use std::iter::FusedIterator;

use super::{op, F26Dot6, GraphicsState};

const MAX_STORAGE_AREAS_TO_RESERVE: usize = 256;
const MAX_FUNCTION_DEFS_TO_RESERVE: usize = 256;

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

    /// An `ELSE` or `EIF` with no corresponding `IF`
    UnexpectedEndOfIfBlock,

    NestedFunctionDefinition,

    UnterminatedIfBlock,

    /// A [Zone](super::graphics_state::Zone) reference that is neither `0` (Twilight Zone) nor `1` (Glyph Zone)
    InvalidZone,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum IterationDecision {
    Break,
    Continue,
}

#[derive(Clone, Debug)]
pub struct Interpreter {
    storage_areas: Box<[u8]>,
    stack: Stack,
    function_definitions: Box<[Option<Vec<u8>>]>,
    is_inside_if: bool,
    graphics_state: GraphicsState,
}

impl Interpreter {
    #[must_use]
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
            stack: Stack::default(),
            function_definitions,
            is_inside_if: false,
            graphics_state: GraphicsState::default(),
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
            Some(op::SRP0) => {
                // Set reference point 0
                self.graphics_state.rp0 = self.stack.pop()?.as_uint32();
            },
            Some(op::SRP1) => {
                // Set reference point 1
                self.graphics_state.rp1 = self.stack.pop()?.as_uint32();
            },
            Some(op::SRP2) => {
                // Set reference point 2
                self.graphics_state.rp2 = self.stack.pop()?.as_uint32();
            },
            Some(op::SZP0) => {
                // Set zone pointer 0
                let n = self.stack.pop()?.as_uint32();
                self.graphics_state.zp0 = Zone::try_from(n)?;
            },
            Some(op::SZP1) => {
                // Set zone pointer 1
                let n = self.stack.pop()?.as_uint32();
                self.graphics_state.zp1 = Zone::try_from(n)?;
            },
            Some(op::SZP2) => {
                // Set zone pointer 2
                let n = self.stack.pop()?.as_uint32();
                self.graphics_state.zp2 = Zone::try_from(n)?;
            },
            Some(op::SZPS) => {
                // Set zone pointers
                let n = self.stack.pop()?.as_uint32();
                let zone = Zone::try_from(n)?;

                self.graphics_state.zp0 = zone;
                self.graphics_state.zp1 = zone;
                self.graphics_state.zp2 = zone;
            },
            Some(op::ELSE) => {
                if !self.is_inside_if {
                    return Err(Error::UnexpectedEndOfIfBlock);
                }

                // If we just found the ELSE it means we're done executing the IF block.
                // Find the corresponding EIF and jump past it.
                let instructions: Instructions<_> =
                    Instructions::new(program.remaining().iter().copied());

                let mut depth = 0;
                for (index, instruction) in instructions.enumerate() {
                    match instruction? {
                        op::IF => depth += 1,
                        op::EIF => {
                            if depth == 0 {
                                program.cursor += index + 1;
                                return Ok(IterationDecision::Continue);
                            } else {
                                depth -= 1;
                            }
                        },
                        _ => {},
                    }
                }

                return Err(Error::UnterminatedIfBlock);
            },
            Some(op::SSW) => {
                // Set single width
                let n_funits = self.stack.pop()?.as_uint32();

                // FIXME: implement this
                _ = n_funits;
            },
            Some(op::CALL) => {
                // Call a previously defined function
                let function_identifier = self.stack.pop()?.as_uint32();

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
                let function_identifier = self.stack.pop()?.as_uint32();

                let function_body = program.consume_function_definition()?.to_owned();
                self.function_definitions[function_identifier as usize] = Some(function_body);
            },
            Some(op::ENDF) => {
                // ENDF (End Function)
                // (only legal if we are within a function definition)
                return Err(Error::UnexpectedEndf);
            },
            Some(op::NPUSHB) => {
                // Push n bytes
                let n = program.next_u8().ok_or(Error::EndOfFileInInstruction)?;

                for _ in 0..n {
                    let byte = program.next_u8().ok_or(Error::EndOfFileInInstruction)?;
                    self.stack.push(byte);
                }
            },
            Some(op::RS) => {
                // RS (Read Storage)
                let address = self.stack.pop()?.as_uint32();
                let storage_value = self
                    .storage_areas
                    .get(address as usize)
                    .ok_or(Error::StorageAddressOutOfRange)?;
                self.stack.push(*storage_value as u32);
            },
            Some(op::MPPEM) => {
                // MPPEM (Measure Pixels Per Em)
            },
            Some(op::DEBUG) => {
                // This instruction is meant to be used during font development,
                // it has no specified effect at any other time
            },
            Some(op::LT) => {
                // Less-than
                let e2 = self.stack.pop()?.as_uint32();
                let e1 = self.stack.pop()?.as_uint32();

                if e1 < e2 {
                    self.stack.push(1);
                } else {
                    self.stack.push(0);
                }
            },
            Some(op::LTEQ) => {
                // Less-than or equal
                let e2 = self.stack.pop()?.as_uint32();
                let e1 = self.stack.pop()?.as_uint32();

                if e1 <= e2 {
                    self.stack.push(1);
                } else {
                    self.stack.push(0);
                }
            },
            Some(op::IF) => {
                let condition = self.stack.pop()?.as_int32() != 0;

                if !condition {
                    // Find the corresponding ELSE (or EIF) instruction, then jump one *past* it
                    let instructions: Instructions<_> =
                        Instructions::new(program.remaining().iter().copied());

                    let mut depth = 0;
                    for (index, instruction) in instructions.enumerate() {
                        match instruction? {
                            op::IF => depth += 1,
                            op::ELSE => {
                                if depth == 0 {
                                    self.is_inside_if = true;
                                    program.cursor += index + 1;
                                    return Ok(IterationDecision::Continue);
                                }
                            },
                            op::EIF => {
                                if depth == 0 {
                                    program.cursor += index + 1;
                                    return Ok(IterationDecision::Continue);
                                } else {
                                    depth -= 1;
                                }
                            },
                            _ => {},
                        }
                    }

                    return Err(Error::UnterminatedIfBlock);
                } else {
                    // Continue as normal, everything else will be handled by other instructions
                    self.is_inside_if = true;
                }
            },

            Some(op::EIF) => {
                if !self.is_inside_if {
                    return Err(Error::UnexpectedEndOfIfBlock);
                }
            },
            Some(op::SDB) => {
                // Set delta base
                self.graphics_state.delta_base = self.stack.pop()?.as_uint32();
            },
            Some(op::SDS) => {
                // Set delta shift
                self.graphics_state.delta_shift = self.stack.pop()?.as_uint32();
            },
            Some(op::ABS) => {
                // Absolute value
                let last_element = self.stack.items.last_mut().ok_or(Error::EmptyStack)?;
                *last_element = last_element.as_f26dot6().abs().into()
            },
            Some(op::AA) => {
                // Adjust Angle (anachronistic)
                self.stack.pop()?;
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
        let mut instructions = Instructions::new(self.remaining().iter().copied());
        let mut depth = 0;
        while let Some(instruction) = instructions.next() {
            let instruction = instruction?;

            if instruction == op::ENDF {
                if depth == 0 {
                    let offset = instructions.offset;
                    let function_body = &self.bytes[self.cursor..self.cursor + offset];
                    self.cursor += offset;
                    return Ok(function_body);
                } else {
                    depth -= 1;
                }
            } else if instruction == op::FDEF {
                depth += 1;
            }
        }
        Err(Error::UnterminatedFunctionDefinition)
    }

    fn next_u8(&mut self) -> Option<u8> {
        let byte = self.bytes.get(self.cursor).copied();
        self.cursor += 1;
        byte
    }
}

/// An iterator over the instructions of a program
///
/// ## Why?
/// Truetype instructions encode their operands as part of the program.
/// For example, the PUSHB instructions push bytes on the stack, and these bytes
/// are stored directly after the instruction.
/// Therefore, simply iterating over the bytes of a program is not sufficient, since
/// operands will be misinterpreted as opcodes.
struct Instructions<I> {
    bytes: I,
    offset: usize,
}

impl<I> Instructions<I> {
    #[must_use]
    fn new(bytes: I) -> Self {
        Self { bytes, offset: 0 }
    }
}

impl<I> Instructions<I>
where
    I: Iterator<Item = u8>,
{
    fn advance_stream(&mut self, n: usize) -> Result<(), Error> {
        if self.bytes.advance_by(n).is_err() {
            return Err(Error::EndOfFileInInstruction);
        }

        self.offset += n;
        Ok(())
    }
}

impl<I> Iterator for Instructions<I>
where
    I: Iterator<Item = u8>,
{
    type Item = Result<u8, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let instruction = self.bytes.next()?;
        self.offset += 1;

        match instruction {
            op::PUSHB_START..=op::PUSHB_END => {
                // If the instruction is a PUSHB then the next few bytes are not instructions
                // but instead values to be pushed onto the stack
                let n_bytes_to_skip = instruction - op::PUSHB_START + 1;
                if let Err(error) = self.advance_stream(n_bytes_to_skip as usize) {
                    return Some(Err(error));
                }
            },
            op::NPUSHB => {
                let Some(n) = self.bytes.next() else {
                    return Some(Err(Error::EndOfFileInInstruction));
                };

                if let Err(error) = self.advance_stream(n as usize) {
                    return Some(Err(error));
                }
            },
            _ => {
                // All other opcodes don't store their data in the instruction stream
            },
        }

        Some(Ok(instruction))
    }
}

impl<I> FusedIterator for Instructions<I> where I: Iterator<Item = u8> {}

#[derive(Clone, Copy, Debug)]
pub enum Zone {
    /// Z0
    Twilight,

    /// Z1
    Glyph,
}

impl TryFrom<u32> for Zone {
    type Error = super::interpreter::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let zone = match value {
            0 => Zone::Twilight,
            1 => Zone::Glyph,
            _ => return Err(Error::InvalidZone),
        };

        Ok(zone)
    }
}

#[derive(Clone, Debug, Default)]
struct Stack {
    items: Vec<StackElement>,
}

impl Stack {
    #[inline]
    fn push<T>(&mut self, item: T)
    where
        T: Into<StackElement>,
    {
        self.items.push(item.into())
    }

    #[inline]
    fn pop(&mut self) -> Result<StackElement, Error> {
        self.items.pop().ok_or(Error::EmptyStack)
    }
}

/// An element on the interpreter [Stack] whose bits can be interpreter in multiple
/// different ways.
///
/// A stack element can be any of the following:
/// * `Eint8` (sign extended 8-bit integer)
/// * `Euint16` (zero extended 16-bit unsigned integer)
/// * `EFWord` (sign extended 16-bit signed integer that describes a quantity in FUnits, the smallest measurable unit in the em space)
/// * `EF2Dot14` (sign extended 16-bit signed fixed number with the low 14 bits representing fraction)
/// * `uint32` (32-bit unsigned integer)
/// * `int32` (32-bit signed integer)
/// * `F26Dot6` (32-bit signed fixed number with the low 6 bits representing fraction)
/// * `StkElt` (any 32 bit quantity)
#[derive(Clone, Copy, Debug)]
struct StackElement(u32);

#[allow(dead_code)]
impl StackElement {
    #[inline]
    #[must_use]
    fn as_bits(&self) -> u32 {
        self.0
    }

    #[inline]
    #[must_use]
    fn as_eint8(&self) -> i8 {
        self.0 as i8
    }

    #[inline]
    #[must_use]
    fn as_euint16(&self) -> u16 {
        self.0 as u16
    }

    #[inline]
    #[must_use]
    fn as_efword(&self) -> i16 {
        self.0 as i16
    }

    #[inline]
    #[must_use]
    fn as_uint32(&self) -> u32 {
        self.0
    }

    #[inline]
    #[must_use]
    fn as_int32(&self) -> i32 {
        self.0 as i32
    }

    #[inline]
    #[must_use]
    fn as_f26dot6(&self) -> F26Dot6 {
        F26Dot6::from_bits(self.as_int32())
    }
}

impl From<u8> for StackElement {
    fn from(value: u8) -> Self {
        Self(value as u32)
    }
}

impl From<u32> for StackElement {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<i32> for StackElement {
    fn from(value: i32) -> Self {
        Self(value as u32)
    }
}

impl From<F26Dot6> for StackElement {
    fn from(value: F26Dot6) -> Self {
        Self(value.bits() as u32)
    }
}
