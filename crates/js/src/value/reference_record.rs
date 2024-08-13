//! <https://262.ecma-international.org/14.0/#sec-reference-record-specification-type>

use crate::runtime::ThrowCompletionOr;

use super::{object::PropertyKey, Value};

/// <https://262.ecma-international.org/14.0/#sec-reference-record-specification-type>
#[derive(Clone, Debug)]
pub struct ReferenceRecord {
    pub base: Value,
    pub referenced_name: String,
}

#[derive(Clone, Debug)]
pub enum ValueOrReference {
    Value(Value),
    Reference(ReferenceRecord),
}

impl ValueOrReference {
    /// <https://262.ecma-international.org/14.0/#sec-getvalue>
    pub fn get_value(&self) -> ThrowCompletionOr<Value> {
        // 1. If V is not a Reference Record, return V.
        let reference_record = match self {
            Self::Value(value) => return Ok(value.clone()),
            Self::Reference(reference) => reference,
        };

        // 2. FIXME: If IsUnresolvableReference(V) is true, throw a ReferenceError exception.

        // 3. If IsPropertyReference(V) is true, then
        //    FIXME: we only have property references

        // a. Let baseObj be ? ToObject(V.[[Base]]).
        let base_obj = reference_record.base.to_object()?;

        // b. FIXME: If IsPrivateReference(V) is true, then

        // c. Return ? baseObj.[[Get]](V.[[ReferencedName]], GetThisValue(V)).
        let property_key = PropertyKey::from(reference_record.referenced_name.clone());
        base_obj.get(&property_key)
    }
}

impl From<Value> for ValueOrReference {
    fn from(value: Value) -> Self {
        Self::Value(value)
    }
}

impl From<ReferenceRecord> for ValueOrReference {
    fn from(value: ReferenceRecord) -> Self {
        Self::Reference(value)
    }
}

impl Default for ValueOrReference {
    fn default() -> Self {
        Self::Value(Value::default())
    }
}
