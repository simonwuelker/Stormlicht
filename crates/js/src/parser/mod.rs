mod declaration;
mod error;
mod expression;
pub mod identifiers;
pub mod literals;
pub mod script;
pub mod tokenizer;

pub use expression::Expression;

use declaration::Declaration;
pub use error::SyntaxError;
pub use script::Script;
pub use tokenizer::Tokenizer;
