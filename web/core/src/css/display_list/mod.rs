//! Provides drawing primitives that can be used to compose a rendered website

mod command;
mod painter;

pub use painter::Painter;
pub use command::Command;