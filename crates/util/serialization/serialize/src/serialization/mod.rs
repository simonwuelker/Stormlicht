mod impls;

pub trait Serialize {
    fn serialize_to<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer;
}

pub trait Serializer {
    type Error;

    type SequenceSerializer<'a>: SerializeSequence<Error = Self::Error>
    where
        Self: 'a;

    type MapSerializer<'a>: SerializeMap<Error = Self::Error>
    where
        Self: 'a;

    type StructSerializer<'a>: SerializeStruct<Error = Self::Error>
    where
        Self: 'a;

    fn serialize_bool(&mut self, value: bool) -> Result<(), Self::Error>;
    fn serialize_string(&mut self, value: &str) -> Result<(), Self::Error>;
    fn serialize_usize(&mut self, value: usize) -> Result<(), Self::Error>;
    fn serialize_sequence<'a, F>(&'a mut self, f: F) -> Result<(), Self::Error>
    where
        F: FnOnce(&mut Self::SequenceSerializer<'a>) -> Result<(), Self::Error>;
    fn serialize_map<'a, F>(&'a mut self, f: F) -> Result<(), Self::Error>
    where
        F: FnOnce(&mut Self::MapSerializer<'a>) -> Result<(), Self::Error>;
    fn serialize_struct<'a, F>(&'a mut self, f: F) -> Result<(), Self::Error>
    where
        F: FnOnce(&mut Self::StructSerializer<'a>) -> Result<(), Self::Error>;
}

pub trait SerializeSequence {
    type Error;

    fn serialize_element<T>(&mut self, element: T) -> Result<(), Self::Error>
    where
        T: Serialize;
}

pub trait SerializeMap {
    type Error;

    fn serialize_key_value_pair<K, V>(&mut self, key: K, value: V) -> Result<(), Self::Error>
    where
        K: Serialize,
        V: Serialize;
}

pub trait SerializeStruct {
    type Error;

    fn serialize_field<T>(&mut self, name: &str, value: T) -> Result<(), Self::Error>
    where
        T: Serialize;
}
