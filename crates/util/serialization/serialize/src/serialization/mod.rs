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

    type TupleVariantSerializer<'a>: SerializeTupleVariant<Error = Self::Error>
    where
        Self: 'a;

    type StructVariantSerializer<'a>: SerializeStructVariant<Error = Self::Error>
    where
        Self: 'a;

    fn serialize_bool(&mut self, value: bool) -> Result<(), Self::Error>;

    fn serialize_string(&mut self, value: &str) -> Result<(), Self::Error>;

    fn serialize_usize(&mut self, value: usize) -> Result<(), Self::Error>;

    fn serialize_sequence<'a>(&'a mut self) -> Result<Self::SequenceSerializer<'a>, Self::Error>;

    fn serialize_map<'a>(&'a mut self) -> Result<Self::MapSerializer<'a>, Self::Error>;

    fn serialize_struct<'a>(&'a mut self) -> Result<Self::StructSerializer<'a>, Self::Error>;

    /// Serialize an enum variant without any associated data.
    ///
    /// Implementations should not usually need to override this.
    fn serialize_enum<'a>(&'a mut self, variant_name: &str) -> Result<(), Self::Error> {
        self.serialize_tuple_enum(variant_name)?.finish()
    }

    fn serialize_tuple_enum<'a>(
        &'a mut self,
        variant_name: &str,
    ) -> Result<Self::TupleVariantSerializer<'a>, Self::Error>;

    fn serialize_struct_enum<'a>(
        &'a mut self,
        variant_name: &str,
    ) -> Result<Self::StructVariantSerializer<'a>, Self::Error>;
}

pub trait SerializeSequence {
    type Error;

    fn serialize_element<T>(&mut self, element: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize;

    fn finish(self) -> Result<(), Self::Error>;
}

pub trait SerializeMap {
    type Error;

    fn serialize_key_value_pair<K, V>(&mut self, key: &K, value: &V) -> Result<(), Self::Error>
    where
        K: ?Sized + Serialize,
        V: ?Sized + Serialize;

    fn finish(self) -> Result<(), Self::Error>;
}

pub trait SerializeStruct {
    type Error;

    fn serialize_field<T>(&mut self, name: &str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize;

    fn finish(self) -> Result<(), Self::Error>;
}

pub trait SerializeStructVariant {
    type Error;

    fn serialize_field<T>(&mut self, name: &str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize;

    fn finish(self) -> Result<(), Self::Error>;
}

pub trait SerializeTupleVariant {
    type Error;

    fn serialize_element<T>(&mut self, element: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize;

    fn finish(self) -> Result<(), Self::Error>;
}
