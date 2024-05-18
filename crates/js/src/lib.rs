#![feature(iter_advance_by, associated_type_defaults)]

mod bytecode;
mod parser;
mod value;

pub use bytecode::{Program, Vm};
pub use value::{Number, Value};
