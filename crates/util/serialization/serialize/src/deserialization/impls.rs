use std::{ascii, collections::HashMap, hash::Hash, marker::PhantomData};

use crate::{
    deserialization::{Error, MapAccess, SequentialAccess},
    Deserialize, Deserializer, Visitor,
};

impl Deserialize for String {
    fn deserialize<D: Deserializer>(deserializer: D) -> Result<Self, D::Error> {
        struct StringVisitor;

        impl Visitor for StringVisitor {
            type Value = String;

            const EXPECTS: &'static str = "a String";

            fn visit_string<E>(&self, value: String) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(value)
            }
        }

        deserializer.deserialize_string(StringVisitor)
    }
}

impl Deserialize for u8 {
    fn deserialize<D: Deserializer>(deserializer: D) -> Result<Self, D::Error> {
        struct U8Visitor;

        impl Visitor for U8Visitor {
            type Value = u8;

            const EXPECTS: &'static str = "a u8";

            fn visit_usize<E>(&self, value: usize) -> Result<Self::Value, E>
            where
                E: Error,
            {
                let value = Self::Value::try_from(value).map_err(|_| E::expected(Self::EXPECTS))?;
                Ok(value)
            }
        }

        deserializer.deserialize_usize(U8Visitor)
    }
}

impl Deserialize for u16 {
    fn deserialize<D: Deserializer>(deserializer: D) -> Result<Self, D::Error> {
        struct U16Visitor;

        impl Visitor for U16Visitor {
            type Value = u16;

            const EXPECTS: &'static str = "a u16";

            fn visit_usize<E>(&self, value: usize) -> Result<Self::Value, E>
            where
                E: Error,
            {
                let value = Self::Value::try_from(value).map_err(|_| E::expected(Self::EXPECTS))?;
                Ok(value)
            }
        }

        deserializer.deserialize_usize(U16Visitor)
    }
}

impl Deserialize for u32 {
    fn deserialize<D: Deserializer>(deserializer: D) -> Result<Self, D::Error> {
        struct U32Visitor;

        impl Visitor for U32Visitor {
            type Value = u32;

            const EXPECTS: &'static str = "a u32";

            fn visit_usize<E>(&self, value: usize) -> Result<Self::Value, E>
            where
                E: Error,
            {
                let value = Self::Value::try_from(value).map_err(|_| E::expected(Self::EXPECTS))?;
                Ok(value)
            }
        }

        deserializer.deserialize_usize(U32Visitor)
    }
}

impl Deserialize for u64 {
    fn deserialize<D: Deserializer>(deserializer: D) -> Result<Self, D::Error> {
        struct U64Visitor;

        impl Visitor for U64Visitor {
            type Value = u64;

            const EXPECTS: &'static str = "a u64";

            fn visit_usize<E>(&self, value: usize) -> Result<Self::Value, E>
            where
                E: Error,
            {
                let value = Self::Value::try_from(value).map_err(|_| E::expected(Self::EXPECTS))?;
                Ok(value)
            }
        }

        deserializer.deserialize_usize(U64Visitor)
    }
}

impl Deserialize for u128 {
    fn deserialize<D: Deserializer>(deserializer: D) -> Result<Self, D::Error> {
        struct U128Visitor;

        impl Visitor for U128Visitor {
            type Value = u128;

            const EXPECTS: &'static str = "a u128";

            fn visit_usize<E>(&self, value: usize) -> Result<Self::Value, E>
            where
                E: Error,
            {
                let value = Self::Value::try_from(value).map_err(|_| E::expected(Self::EXPECTS))?;
                Ok(value)
            }
        }

        deserializer.deserialize_usize(U128Visitor)
    }
}

impl Deserialize for usize {
    fn deserialize<D: Deserializer>(deserializer: D) -> Result<Self, D::Error> {
        struct UsizeVisitor;

        impl Visitor for UsizeVisitor {
            type Value = usize;

            const EXPECTS: &'static str = "a usize";

            fn visit_usize<E>(&self, value: usize) -> Result<Self::Value, E>
            where
                E: Error,
            {
                let value = Self::Value::try_from(value).map_err(|_| E::expected(Self::EXPECTS))?;
                Ok(value)
            }
        }

        deserializer.deserialize_usize(UsizeVisitor)
    }
}

impl<T: Deserialize> Deserialize for Vec<T> {
    fn deserialize<D: Deserializer>(deserializer: D) -> Result<Self, D::Error> {
        struct VecVisitor<T> {
            marker: PhantomData<T>,
        }

        impl<T> Visitor for VecVisitor<T>
        where
            T: Deserialize,
        {
            type Value = Vec<T>;

            const EXPECTS: &'static str = "a sequence of values";

            fn visit_sequence<S>(&self, mut sequence: S) -> Result<Self::Value, S::Error>
            where
                S: SequentialAccess,
            {
                let mut result = vec![];
                loop {
                    let element = sequence.next_element()?;

                    match element {
                        Some(element) => result.push(element),
                        None => break,
                    }
                }
                Ok(result)
            }
        }

        deserializer.deserialize_sequence(VecVisitor {
            marker: PhantomData,
        })
    }
}

impl<K, V> Deserialize for HashMap<K, V>
where
    K: Deserialize + Hash + Eq,
    V: Deserialize,
{
    fn deserialize<D: Deserializer>(deserializer: D) -> Result<Self, D::Error> {
        struct HashMapVisitor<K, V> {
            marker: PhantomData<(K, V)>,
        }

        impl<K, V> Visitor for HashMapVisitor<K, V>
        where
            K: Deserialize + Hash + Eq,
            V: Deserialize,
        {
            type Value = HashMap<K, V>;

            const EXPECTS: &'static str = "a map";

            fn visit_map<M>(&self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess,
            {
                let mut result = HashMap::new();

                loop {
                    let Some(key) = map.next_key()? else {
                        break;
                    };

                    let value = map.next_value()?;
                    result.insert(key, value);
                }

                Ok(result)
            }
        }

        deserializer.deserialize_map(HashMapVisitor {
            marker: PhantomData,
        })
    }
}

impl Deserialize for ascii::Char {
    fn deserialize<D: Deserializer>(deserializer: D) -> Result<Self, D::Error> {
        let byte = u8::deserialize(deserializer)?;
        let c = Self::from_u8(byte)
            .ok_or_else(|| D::Error::expected("a byte in the ascii value range"))?;

        Ok(c)
    }
}

impl<T> Deserialize for Option<T>
where
    T: Deserialize,
{
    fn deserialize<D: Deserializer>(deserializer: D) -> Result<Self, D::Error> {
        struct OptionVisitor<T> {
            marker: PhantomData<T>,
        }

        impl<T> Visitor for OptionVisitor<T>
        where
            T: Deserialize,
        {
            type Value = Option<T>;

            // TODO: Can we make this a bit more helpful? (:
            const EXPECTS: &'static str = concat!("Either something or nothing");

            fn visit_none<E>(&self) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(None)
            }

            fn visit_some<D>(&self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer,
            {
                let value = T::deserialize(deserializer)?;

                Ok(Some(value))
            }
        }

        deserializer.deserialize_option(OptionVisitor {
            marker: PhantomData,
        })
    }
}
