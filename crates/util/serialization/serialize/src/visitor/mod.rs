use crate::{
    deserialization::{EnumAccess, Error, MapAccess, SequentialAccess},
    Deserializer,
};

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

    fn visit_none<E>(&self) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Err(E::expected(Self::EXPECTS))
    }

    fn visit_some<D>(&self, value: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer,
    {
        _ = value;

        Err(D::Error::expected(Self::EXPECTS))
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

    fn visit_enum<E>(&self, value: E) -> Result<Self::Value, E::Error>
    where
        E: EnumAccess,
    {
        _ = value;
        Err(E::Error::expected(Self::EXPECTS))
    }
}
