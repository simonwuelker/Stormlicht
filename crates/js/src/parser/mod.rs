mod declaration;
mod error;
mod expression;
mod functions_and_classes;
pub mod identifiers;
pub mod literals;
pub mod script;
pub mod tokenizer;

pub use expression::Expression;

use declaration::Declaration;
pub use error::SyntaxError;
pub use script::Script;
pub use tokenizer::Tokenizer;
