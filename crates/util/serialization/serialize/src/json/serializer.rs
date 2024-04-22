use std::io::Write;

use crate::serialization::{SerializeMap, SerializeSequence, SerializeStruct, Serializer};

pub struct JsonSerializer<W> {
    writer: W,
}

impl<W> JsonSerializer<W> where W: Write {}

impl<W> Serializer for JsonSerializer<W>
where
    W: Write,
{
    type Error = std::io::Error;
    type SequenceSerializer<'a> = SequenceSerializer<'a, W> where Self: 'a;
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

    fn serialize_sequence<'a, F>(&'a mut self, f: F) -> Result<(), Self::Error>
    where
        F: FnOnce(&mut Self::SequenceSerializer<'a>) -> Result<(), Self::Error>,
    {
        let mut sequence_serializer = SequenceSerializer(self);
        sequence_serializer.start()?;
        f(&mut sequence_serializer)?;
        sequence_serializer.end()?;

        Ok(())
    }

    fn serialize_map<'a, F>(&'a mut self, f: F) -> Result<(), Self::Error>
    where
        F: FnOnce(&mut Self::MapSerializer<'a>) -> Result<(), Self::Error>,
    {
        let mut map_serializer = MapSerializer(self);
        map_serializer.start()?;
        f(&mut map_serializer)?;
        map_serializer.end()?;

        Ok(())
    }

    fn serialize_struct<'a, F>(&'a mut self, f: F) -> Result<(), Self::Error>
    where
        F: FnOnce(&mut Self::StructSerializer<'a>) -> Result<(), Self::Error>,
    {
        let mut struct_serializer = StructSerializer(MapSerializer(self));
        struct_serializer.0.start()?;
        f(&mut struct_serializer)?;
        struct_serializer.0.end()?;

        Ok(())
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
}

impl<'a, W> SequenceSerializer<'a, W>
where
    W: Write,
{
    fn start(&mut self) -> Result<(), std::io::Error> {
        write!(self.0.writer, "[")
    }

    fn end(&mut self) -> Result<(), std::io::Error> {
        write!(self.0.writer, "]")
    }
}

impl<'a, W> MapSerializer<'a, W>
where
    W: Write,
{
    fn start(&mut self) -> Result<(), std::io::Error> {
        write!(self.0.writer, "{{")
    }

    fn end(&mut self) -> Result<(), std::io::Error> {
        write!(self.0.writer, "}}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_struct() {
        let mut result = Vec::new();
        let mut serializer = JsonSerializer {
            writer: &mut result,
        };
        serializer.serialize_sequence(|_| Ok(())).unwrap();
        serializer.serialize_bool(false).unwrap();
        todo!()
    }
}
