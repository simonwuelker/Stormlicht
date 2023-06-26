use std::collections::HashMap;

#[cfg(feature = "derive")]
pub use serialize_derive::Deserialize;

mod compound_types;
pub mod json;

pub use compound_types::{Map, Sequence};

pub struct StructInfo;

impl StructInfo {}

pub trait Deserialize: Sized {
    fn deserialize<D: Deserializer>(deserializer: &mut D) -> Result<Self, D::Error>;
}

pub trait Deserializer {
    type Error;

    fn deserialize_field<T: Deserialize>(&mut self, name: &str) -> Result<T, Self::Error>;
    fn deserialize_sequence<S: Sequence>(&mut self) -> Result<S, Self::Error>
    where
        S::Item: Deserialize;
    fn deserialize_map<M: Map>(&mut self) -> Result<M, Self::Error>
    where
        M::Value: Deserialize;
    fn start_struct(&mut self) -> Result<(), Self::Error>;
    fn end_struct(&mut self) -> Result<(), Self::Error>;

    fn deserialize_string(&mut self) -> Result<String, Self::Error>;
    fn deserialize_usize(&mut self) -> Result<usize, Self::Error>;
}

impl<T: Deserialize> Deserialize for Vec<T> {
    fn deserialize<D: Deserializer>(deserializer: &mut D) -> Result<Self, D::Error> {
        deserializer.deserialize_sequence()
    }
}

impl<T: Deserialize> Deserialize for HashMap<String, T> {
    fn deserialize<D: Deserializer>(deserializer: &mut D) -> Result<Self, D::Error> {
        deserializer.deserialize_map()
    }
}

macro_rules! deserialize_int {
    ($t: ident) => {
        impl Deserialize for $t {
            fn deserialize<D: Deserializer>(deserializer: &mut D) -> Result<Self, D::Error> {
                deserializer.deserialize_usize().map(|v| v as $t)
            }
        }
    };
}

deserialize_int!(u8);
deserialize_int!(u16);
deserialize_int!(u32);
deserialize_int!(u64);
deserialize_int!(u128);
deserialize_int!(usize);

impl Deserialize for String {
    fn deserialize<D: Deserializer>(deserializer: &mut D) -> Result<Self, D::Error> {
        deserializer.deserialize_string()
    }
}
