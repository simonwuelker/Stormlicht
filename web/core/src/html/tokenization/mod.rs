mod error_handler;
mod named_character_reference;
mod token;
mod tokenizer;

pub use error_handler::{HtmlParseError, IgnoreParseErrors, ParseErrorHandler};
pub use named_character_reference::lookup_character_reference;
pub use token::{Doctype, TagData, Token};
pub use tokenizer::{Tokenizer, TokenizerState};
