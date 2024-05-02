#![feature(ascii_char)]

#[cfg(feature = "derive")]
pub use serialize_derive::{Deserialize, Serialize};

mod compound_types;
pub mod deserialization;
pub mod serialization;
mod visitor;

pub use compound_types::{Map, Sequence};
pub use deserialization::{Deserialize, Deserializer};
pub use serialization::{Serialize, Serializer};
pub use visitor::Visitor;
