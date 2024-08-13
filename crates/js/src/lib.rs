#![feature(iter_advance_by, associated_type_defaults, assert_matches)]

mod bytecode;
mod compiler;
mod parser;
mod runtime;
mod value;

pub use bytecode::{Program, Vm};
pub use value::{Number, Value};
