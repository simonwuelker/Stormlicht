mod error;
mod expression;
mod functions_and_classes;
pub mod identifiers;
pub mod literals;
pub mod script;
mod statements_and_declarations;
pub mod tokenizer;

pub use expression::Expression;

pub use error::SyntaxError;
pub use script::Script;
pub use tokenizer::Tokenizer;