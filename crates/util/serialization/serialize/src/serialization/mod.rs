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

    fn serialize_sequence<'a>(&'a mut self) -> Result<Self::SequenceSerializer<'a>, Self::Error>;

    fn serialize_map<'a>(&'a mut self) -> Result<Self::MapSerializer<'a>, Self::Error>;

    fn serialize_struct<'a>(&'a mut self) -> Result<Self::StructSerializer<'a>, Self::Error>;
}

pub trait SerializeSequence {
    type Error;

    fn serialize_element<T>(&mut self, element: &T) -> Result<(), Self::Error>
    where
        T: Serialize;

    fn finish(self) -> Result<(), Self::Error>;
}

pub trait SerializeMap {
    type Error;

    fn serialize_key_value_pair<K, V>(&mut self, key: &K, value: &V) -> Result<(), Self::Error>
    where
        K: Serialize,
        V: Serialize;

    fn finish(self) -> Result<(), Self::Error>;
}

pub trait SerializeStruct {
    type Error;

    fn serialize_field<T>(&mut self, name: &str, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize;

    fn finish(self) -> Result<(), Self::Error>;
}
