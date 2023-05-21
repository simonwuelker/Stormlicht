#![feature(exclusive_range_pattern, associated_type_defaults, let_chains)]

mod browsing_context;
pub mod css;
pub mod dom;
pub mod infra;
pub mod tokenization;
pub mod treebuilding;

pub use browsing_context::BrowsingContext;
pub use treebuilding::parser::Parser;
