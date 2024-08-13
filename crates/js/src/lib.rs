#![feature(iter_advance_by, associated_type_defaults, assert_matches)]

mod compiler;
mod parser;
mod runtime;
mod value;

pub use runtime::{Executable, Vm};
pub use value::{Number, Value};
