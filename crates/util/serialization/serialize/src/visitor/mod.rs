mod bool_visitor;

use crate::deserialization::{Error, MapAccess, SequentialAccess};

pub trait Visitor {
    type Value;

    const EXPECTS: &'static str;

    fn visit_bool<E>(&self, value: bool) -> Result<Self::Value, E>
    where
        E: Error,
    {
        _ = value;
        Err(E::expected(Self::EXPECTS))
    }

    fn visit_usize<E>(&self, value: usize) -> Result<Self::Value, E>
    where
        E: Error,
    {
        _ = value;
        Err(E::expected(Self::EXPECTS))
    }

    fn visit_string<E>(&self, value: String) -> Result<Self::Value, E>
    where
        E: Error,
    {
        _ = value;
        Err(E::expected(Self::EXPECTS))
    }

    fn visit_sequence<S>(&self, value: S) -> Result<Self::Value, S::Error>
    where
        S: SequentialAccess,
    {
        _ = value;
        Err(S::Error::expected(Self::EXPECTS))
    }

    fn visit_map<M>(&self, value: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess,
    {
        _ = value;
        Err(M::Error::expected(Self::EXPECTS))
    }
}
