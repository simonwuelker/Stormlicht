#![feature(iter_advance_by)]

mod deserializer;
mod serializer;

pub use deserializer::{JsonDeserializer, JsonError};
pub use serializer::{JsonSerializer, MapSerializer, SequenceSerializer};
