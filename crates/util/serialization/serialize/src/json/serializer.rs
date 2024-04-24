use std::{fmt, fmt::Write};

use crate::{
    serialization::{SerializeMap, SerializeSequence, SerializeStruct, Serializer},
    Serialize,
};

pub struct JsonSerializer<W> {
    writer: W,
}

impl JsonSerializer<String> {
    pub fn serialize_to_string<T>(value: T) -> Result<String, fmt::Error>
    where
        T: Serialize,
    {
        let mut serializer = Self {
            writer: String::new(),
        };
        value.serialize_to(&mut serializer)?;

        Ok(serializer.writer)
    }
}

impl<W> Serializer for JsonSerializer<W>
where
    W: Write,
{
    type Error = fmt::Error;
    type SequenceSerializer<'a> = SequenceSerializer<'a, W> where Self: 'a, W: 'a;
    type MapSerializer<'a> = MapSerializer<'a, W> where Self: 'a;
    type StructSerializer<'a> = StructSerializer<'a, W> where Self: 'a;

    fn serialize_bool(&mut self, value: bool) -> Result<(), Self::Error> {
        match value {
            true => write!(self.writer, "true")?,
            false => write!(self.writer, "false")?,
        }

        Ok(())
    }

    fn serialize_string(&mut self, value: &str) -> Result<(), Self::Error> {
        write!(self.writer, "{value:?}")
    }

    fn serialize_usize(&mut self, value: usize) -> Result<(), Self::Error> {
        write!(self.writer, "{value}")
    }

    fn serialize_sequence<'a>(&'a mut self) -> Result<Self::SequenceSerializer<'a>, Self::Error> {
        write!(self.writer, "[")?;
        let sequence_serializer = SequenceSerializer(self);

        Ok(sequence_serializer)
    }

    fn serialize_map<'a>(&'a mut self) -> Result<Self::MapSerializer<'a>, Self::Error> {
        write!(self.writer, "{{")?;
        let map_serializer = MapSerializer(self);

        Ok(map_serializer)
    }

    fn serialize_struct<'a>(&'a mut self) -> Result<Self::StructSerializer<'a>, Self::Error> {
        write!(self.writer, "{{")?;
        let struct_serializer = StructSerializer(MapSerializer(self));

        Ok(struct_serializer)
    }
}

pub struct SequenceSerializer<'a, W>(&'a mut JsonSerializer<W>);

pub struct MapSerializer<'a, W>(&'a mut JsonSerializer<W>);

// Structs are just maps with string keys
pub struct StructSerializer<'a, W>(MapSerializer<'a, W>);

impl<'a, W> SerializeSequence for SequenceSerializer<'a, W>
where
    W: Write,
{
    type Error = <JsonSerializer<W> as Serializer>::Error;

    fn serialize_element<T>(&mut self, element: &T) -> Result<(), Self::Error>
    where
        T: crate::serialization::Serialize,
    {
        element.serialize_to(self.0)?;
        write!(self.0.writer, ",")?;
        Ok(())
    }

    fn finish(self) -> Result<(), Self::Error> {
        write!(self.0.writer, "]")
    }
}

impl<'a, W> SerializeMap for MapSerializer<'a, W>
where
    W: Write,
{
    type Error = <JsonSerializer<W> as Serializer>::Error;

    fn serialize_key_value_pair<K, V>(&mut self, key: &K, value: &V) -> Result<(), Self::Error>
    where
        K: crate::serialization::Serialize,
        V: crate::serialization::Serialize,
    {
        key.serialize_to(self.0)?;
        write!(self.0.writer, ":")?;
        value.serialize_to(self.0)?;
        write!(self.0.writer, ",")?;
        Ok(())
    }

    fn finish(self) -> Result<(), Self::Error> {
        write!(self.0.writer, "}}")
    }
}

impl<'a, W> SerializeStruct for StructSerializer<'a, W>
where
    W: Write,
{
    type Error = <JsonSerializer<W> as Serializer>::Error;

    fn serialize_field<T>(&mut self, name: &str, value: &T) -> Result<(), Self::Error>
    where
        T: crate::serialization::Serialize,
    {
        self.0.serialize_key_value_pair(&name, value)
    }

    fn finish(self) -> Result<(), Self::Error> {
        self.0.finish()
    }
}
