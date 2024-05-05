mod impls;

use crate::Visitor;

/// A error emitted by a [Deserializer]
pub trait Error {
    fn expected(expectation: &'static str) -> Self;
    fn unknown_field(field: String) -> Self;
    fn unknown_variant(name: String) -> Self;
    fn missing_field(field: &'static str) -> Self;
}

pub trait Deserialize: Sized {
    fn deserialize<D: Deserializer>(deserializer: D) -> Result<Self, D::Error>;
}

pub trait Deserializer {
    type Error: Error;

    fn deserialize_sequence<V: Visitor>(self, visitor: V) -> Result<V::Value, Self::Error>;

    fn deserialize_map<V: Visitor>(self, visitor: V) -> Result<V::Value, Self::Error>;

    fn deserialize_struct<V: Visitor>(self, visitor: V) -> Result<V::Value, Self::Error>;

    fn deserialize_bool<V: Visitor>(self, visitor: V) -> Result<V::Value, Self::Error>;

    fn deserialize_string<V: Visitor>(self, visitor: V) -> Result<V::Value, Self::Error>;

    fn deserialize_usize<V: Visitor>(self, visitor: V) -> Result<V::Value, Self::Error>;

    fn deserialize_option<V: Visitor>(self, visitor: V) -> Result<V::Value, Self::Error>;

    fn deserialize_enum<V: Visitor>(self, visitor: V) -> Result<V::Value, Self::Error>;
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

pub trait EnumAccess {
    type Error: Error;
    type Variant: EnumVariantAccess<Error = Self::Error>;

    fn variant<V>(self) -> Result<(V, Self::Variant), Self::Error>
    where
        V: Deserialize;
}

pub trait EnumVariantAccess {
    type Error: Error;

    fn unit_variant(self) -> Result<(), Self::Error>;

    fn struct_variant<V: Visitor>(self, visitor: V) -> Result<V::Value, Self::Error>;

    fn tuple_variant<V: Visitor>(self, visitor: V) -> Result<V::Value, Self::Error>;
}
