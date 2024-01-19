mod declaration;
mod error;
mod expression;
pub mod grammar;
pub mod identifiers;
pub mod literals;
pub mod tokenizer;

pub use expression::Expression;

use declaration::Declaration;
pub use error::SyntaxError;
pub use grammar::Script;
pub use tokenizer::Tokenizer;
