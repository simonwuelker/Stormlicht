use crate::Visitor;

struct BoolVisitor;

impl Visitor for BoolVisitor {
    type Value = bool;

    const EXPECTS: &'static str = "true or false";

    fn visit_bool<E>(&self, value: bool) -> Result<Self::Value, E>
    where
        E: crate::deserialization::Error,
    {
        Ok(value)
    }
}
