use std::{ascii, collections::HashMap, hash::Hash, marker::PhantomData, net};

use crate::{
    deserialization::{EnumAccess, EnumVariantAccess, Error, MapAccess, SequentialAccess},
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

impl Deserialize for bool {
    fn deserialize<D: Deserializer>(deserializer: D) -> Result<Self, D::Error> {
        struct BoolVisitor;

        impl Visitor for BoolVisitor {
            type Value = bool;

            const EXPECTS: &'static str = "Either true or false";

            fn visit_bool<E>(&self, value: bool) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(value)
            }
        }

        deserializer.deserialize_bool(BoolVisitor)
    }
}

impl<T, const N: usize> Deserialize for [T; N]
where
    T: Deserialize,
{
    fn deserialize<D: Deserializer>(deserializer: D) -> Result<Self, D::Error> {
        // FIXME: There's probably a way to do this without heap allocations
        let items = <Vec<T> as Deserialize>::deserialize(deserializer)?;

        let array = items
            .try_into()
            .map_err(|_| D::Error::expected("the correct number of elements"))?;

        Ok(array)
    }
}

impl Deserialize for net::Ipv4Addr {
    fn deserialize<D: Deserializer>(deserializer: D) -> Result<Self, D::Error> {
        let octets = <[u8; 4] as Deserialize>::deserialize(deserializer)?;

        let ipv4 = Self::new(octets[0], octets[1], octets[2], octets[3]);

        Ok(ipv4)
    }
}

impl Deserialize for net::Ipv6Addr {
    fn deserialize<D: Deserializer>(deserializer: D) -> Result<Self, D::Error> {
        let segments = <[u16; 8] as Deserialize>::deserialize(deserializer)?;

        let ipv4 = Self::new(
            segments[0],
            segments[1],
            segments[2],
            segments[3],
            segments[4],
            segments[5],
            segments[6],
            segments[7],
        );

        Ok(ipv4)
    }
}

impl Deserialize for net::IpAddr {
    fn deserialize<D: Deserializer>(deserializer: D) -> Result<Self, D::Error> {
        enum Variant {
            V4,
            V6,
        }

        impl Deserialize for Variant {
            fn deserialize<D: Deserializer>(deserializer: D) -> Result<Self, D::Error> {
                struct VariantVisitor;

                impl Visitor for VariantVisitor {
                    type Value = Variant;

                    const EXPECTS: &'static str = "v4 or v6";

                    fn visit_string<E>(&self, value: String) -> Result<Self::Value, E>
                    where
                        E: Error,
                    {
                        let variant = match value.as_str() {
                            "v4" => Variant::V4,
                            "v6" => Variant::V6,
                            _ => return Err(E::unknown_variant(value)),
                        };

                        Ok(variant)
                    }
                }

                deserializer.deserialize_string(VariantVisitor)
            }
        }

        struct IpAddrVisitor;

        impl Visitor for IpAddrVisitor {
            type Value = net::IpAddr;

            const EXPECTS: &'static str = "An IP address";

            fn visit_enum<E>(&self, value: E) -> Result<Self::Value, E::Error>
            where
                E: EnumAccess,
            {
                let ip = match value.variant()? {
                    (Variant::V4, variant_data) => {
                        let ipv4 = variant_data.newtype_variant()?;

                        net::IpAddr::V4(ipv4)
                    },
                    (Variant::V6, variant_data) => {
                        let ipv4 = variant_data.newtype_variant()?;

                        net::IpAddr::V6(ipv4)
                    },
                };

                Ok(ip)
            }
        }

        deserializer.deserialize_enum(IpAddrVisitor)
    }
}
