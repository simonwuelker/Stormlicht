mod error;
mod expressions;
mod functions_and_classes;
pub mod identifiers;
pub mod literals;
pub mod script;
mod statements_and_declarations;
pub mod tokenization;

pub use error::SyntaxError;
pub use script::Script;
