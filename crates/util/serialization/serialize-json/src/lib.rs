#![feature(iter_advance_by)]

mod deserializer;
mod serializer;
mod value;

pub use deserializer::{JsonDeserializer, JsonError};
pub use serializer::{JsonSerializer, MapSerializer, SequenceSerializer};
pub use value::Value;

#[cfg(test)]
mod tests {
    use serialize::{Deserialize, Serialize};

    use super::*;

    fn serialize_deserialize<T>(value: T) -> T
    where
        T: Serialize + Deserialize,
    {
        let serialized = JsonSerializer::serialize_to_string(value).unwrap();
        let mut deserializer = JsonDeserializer::new(&serialized);
        let deserialized = T::deserialize(&mut deserializer).unwrap();

        deserialized
    }

    #[test]
    fn serialize_deserialize_string() {
        let s = "foo".to_string();
        assert_eq!(serialize_deserialize(s.clone()), s);
    }

    #[test]
    fn serialize_deserialize_option() {
        let value: Option<usize> = Some(3);

        assert_eq!(serialize_deserialize(value), value)
    }
}
