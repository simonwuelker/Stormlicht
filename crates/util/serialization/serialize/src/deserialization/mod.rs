mod impls;

use crate::Visitor;

/// A error emitted by a [Deserializer]
pub trait Error {
    fn expected(expectation: &'static str) -> Self;
    fn unknown_field(field: String) -> Self;
    fn missing_field() -> Self;
}

pub trait Deserialize: Sized {
    fn deserialize<D: Deserializer>(deserializer: &mut D) -> Result<Self, D::Error>;
}

pub trait Deserializer {
    type Error: Error;

    fn deserialize_sequence<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;
    fn deserialize_map<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;
    fn deserialize_struct<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;
    fn deserialize_string<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;
    fn deserialize_usize<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;
}

pub trait SequentialAccess {
    type Error: Error;

    fn next_element<T>(&mut self) -> Result<Option<T>, Self::Error>
    where
        T: Deserialize;
}

pub trait MapAccess {
    type Error: Error;

    fn next_key<K>(&mut self) -> Result<Option<K>, Self::Error>
    where
        K: Deserialize;

    fn next_value<V>(&mut self) -> Result<V, Self::Error>
    where
        V: Deserialize;
}
