use std::collections::HashMap;

use serialize::{
    deserialization::{Error, MapAccess, SequentialAccess},
    serialization::{SerializeMap, SerializeSequence},
    Deserialize, Deserializer, Serialize, Visitor,
};

#[derive(Clone, Debug)]
pub enum Value {
    String(String),
    Integer(usize),
    Boolean(bool),
    List(Vec<Value>),
    Map(HashMap<String, Value>),
    Null,
}

impl Serialize for Value {
    fn serialize_to<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: serialize::Serializer,
    {
        match self {
            Self::String(s) => serializer.serialize_string(s),
            Self::Integer(int) => serializer.serialize_usize(*int),
            Self::Boolean(boolean) => serializer.serialize_bool(*boolean),
            Self::List(list) => {
                let mut sequence = serializer.serialize_sequence()?;
                for element in list {
                    sequence.serialize_element(element)?;
                }
                sequence.finish()?;
                Ok(())
            },
            Self::Map(hashmap) => {
                let mut map = serializer.serialize_map()?;
                for (key, value) in hashmap {
                    map.serialize_key_value_pair(key, value)?;
                }
                map.finish()?;
                Ok(())
            },
            // FIXME: we should not need a bogus type parameter here
            Self::Null => serializer.serialize_option::<String>(&None),
        }
    }
}

impl Deserialize for Value {
    fn deserialize<D: Deserializer>(deserializer: D) -> Result<Self, D::Error> {
        struct ValueVisitor;

        impl Visitor for ValueVisitor {
            type Value = Value;

            const EXPECTS: &'static str = "Any JSON value";

            fn visit_bool<E>(&self, value: bool) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Value::Boolean(value))
            }

            fn visit_string<E>(&self, value: String) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Value::String(value))
            }

            fn visit_usize<E>(&self, value: usize) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Value::Integer(value))
            }

            fn visit_map<M>(&self, mut value: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess,
            {
                let mut map = HashMap::default();
                while let Some(key) = value.next_key()? {
                    let value = value.next_value()?;

                    map.insert(key, value);
                }

                Ok(Value::Map(map))
            }

            fn visit_sequence<S>(&self, mut value: S) -> Result<Self::Value, S::Error>
            where
                S: SequentialAccess,
            {
                let mut sequence = Vec::default();

                while let Some(element) = value.next_element()? {
                    sequence.push(element);
                }

                Ok(Value::List(sequence))
            }

            fn visit_none<E>(&self) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Value::Null)
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

impl Value {
    #[must_use]
    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    // FIXME: we may want to not expose the concrete type here
    #[must_use]
    pub fn as_map(&self) -> Option<&HashMap<String, Self>> {
        match self {
            Self::Map(map) => Some(map),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_list(&self) -> Option<impl Iterator<Item = &Self>> {
        match self {
            Self::List(list) => Some(list.iter()),
            _ => None,
        }
    }
}
