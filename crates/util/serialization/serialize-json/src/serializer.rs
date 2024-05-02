use std::{fmt, fmt::Write};

use serialize::{
    serialization::{
        SerializeMap, SerializeSequence, SerializeStruct, SerializeStructVariant,
        SerializeTupleVariant, Serializer,
    },
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

    type StructVariantSerializer<'a> = StructVariantSerializer<'a, W> where Self: 'a;

    type TupleVariantSerializer<'a> = TupleVariantSerializer<'a, W> where Self: 'a;

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
        let sequence_serializer = SequenceSerializer(CommaSeparatedSequence::new(self));

        Ok(sequence_serializer)
    }

    fn serialize_map<'a>(&'a mut self) -> Result<Self::MapSerializer<'a>, Self::Error> {
        write!(self.writer, "{{")?;
        let map_serializer = MapSerializer(CommaSeparatedSequence::new(self));

        Ok(map_serializer)
    }

    fn serialize_struct<'a>(&'a mut self) -> Result<Self::StructSerializer<'a>, Self::Error> {
        write!(self.writer, "{{")?;
        let struct_serializer = StructSerializer(MapSerializer(CommaSeparatedSequence::new(self)));

        Ok(struct_serializer)
    }

    fn serialize_struct_enum<'a>(
        &'a mut self,
        variant_name: &str,
    ) -> Result<Self::StructVariantSerializer<'a>, Self::Error> {
        write!(self.writer, "{{{variant_name:?}:{{")?;

        let struct_variant_serializer = StructVariantSerializer(CommaSeparatedSequence::new(self));

        Ok(struct_variant_serializer)
    }

    fn serialize_tuple_enum<'a>(
        &'a mut self,
        variant_name: &str,
    ) -> Result<Self::TupleVariantSerializer<'a>, Self::Error> {
        write!(self.writer, "{{{variant_name:?}:[")?;

        let struct_variant_serializer = TupleVariantSerializer(CommaSeparatedSequence::new(self));

        Ok(struct_variant_serializer)
    }
}

struct CommaSeparatedSequence<'a, W> {
    serializer: &'a mut JsonSerializer<W>,
    is_first_element: bool,
}

pub struct SequenceSerializer<'a, W>(CommaSeparatedSequence<'a, W>);

pub struct MapSerializer<'a, W>(CommaSeparatedSequence<'a, W>);

// Structs are just maps with string keys
pub struct StructSerializer<'a, W>(MapSerializer<'a, W>);

pub struct StructVariantSerializer<'a, W>(CommaSeparatedSequence<'a, W>);

pub struct TupleVariantSerializer<'a, W>(CommaSeparatedSequence<'a, W>);

impl<'a, W> SerializeSequence for SequenceSerializer<'a, W>
where
    W: Write,
{
    type Error = <JsonSerializer<W> as Serializer>::Error;

    fn serialize_element<T>(&mut self, element: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.0.start_element()?;
        element.serialize_to(self.0.serializer)?;
        Ok(())
    }

    fn finish(mut self) -> Result<(), Self::Error> {
        write!(self.0.writer(), "]")
    }
}

impl<'a, W> SerializeMap for MapSerializer<'a, W>
where
    W: Write,
{
    type Error = <JsonSerializer<W> as Serializer>::Error;

    fn serialize_key_value_pair<K, V>(&mut self, key: &K, value: &V) -> Result<(), Self::Error>
    where
        K: Serialize,
        V: Serialize,
    {
        self.0.start_element()?;

        key.serialize_to(self.0.serializer)?;
        write!(self.0.writer(), ":")?;
        value.serialize_to(self.0.serializer)?;
        Ok(())
    }

    fn finish(mut self) -> Result<(), Self::Error> {
        write!(self.0.writer(), "}}")
    }
}

impl<'a, W> SerializeStruct for StructSerializer<'a, W>
where
    W: Write,
{
    type Error = <JsonSerializer<W> as Serializer>::Error;

    fn serialize_field<T>(&mut self, name: &str, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.0.serialize_key_value_pair(&name, value)
    }

    fn finish(self) -> Result<(), Self::Error> {
        self.0.finish()
    }
}

impl<'a, W> SerializeStructVariant for StructVariantSerializer<'a, W>
where
    W: Write,
{
    type Error = <JsonSerializer<W> as Serializer>::Error;

    fn serialize_field<T>(&mut self, name: &str, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.0.start_element()?;
        name.serialize_to(self.0.serializer)?;
        write!(self.0.serializer.writer, ":")?;
        value.serialize_to(self.0.serializer)?;

        Ok(())
    }

    fn finish(self) -> Result<(), Self::Error> {
        write!(self.0.serializer.writer, "}}}}")
    }
}

impl<'a, W> SerializeTupleVariant for TupleVariantSerializer<'a, W>
where
    W: Write,
{
    type Error = <JsonSerializer<W> as Serializer>::Error;

    fn serialize_element<T>(&mut self, element: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.0.start_element()?;
        element.serialize_to(self.0.serializer)?;

        Ok(())
    }

    fn finish(self) -> Result<(), Self::Error> {
        write!(self.0.serializer.writer, "]}}")
    }
}

impl<'a, W> CommaSeparatedSequence<'a, W>
where
    W: Write,
{
    #[must_use]
    fn new(serializer: &'a mut JsonSerializer<W>) -> Self {
        Self {
            serializer,
            is_first_element: true,
        }
    }

    fn start_element(&mut self) -> Result<(), <JsonSerializer<W> as Serializer>::Error> {
        if self.is_first_element {
            // There is no comma before the first element
            self.is_first_element = false;
            Ok(())
        } else {
            write!(self.serializer.writer, ",")
        }
    }

    /// Convenience function to access the underlying writer
    #[must_use]
    fn writer(&mut self) -> &mut W {
        &mut self.serializer.writer
    }
}
