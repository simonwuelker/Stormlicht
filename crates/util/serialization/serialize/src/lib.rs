#![feature(iter_advance_by)]

#[cfg(feature = "derive")]
pub use serialize_derive::Deserialize;

mod compound_types;
pub mod deserialization;
pub mod json;
mod visitor;

pub use compound_types::{Map, Sequence};
pub use deserialization::{Deserialize, Deserializer};
pub use visitor::Visitor;
