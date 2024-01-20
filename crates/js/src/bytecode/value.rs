use super::{Exception, ThrowCompletionOr};

#[derive(Clone, Debug, Default)]
pub enum Value {
    /// <https://262.ecma-international.org/14.0/#sec-ecmascript-language-types-undefined-type>
    #[default]
    Undefined,

    /// <https://262.ecma-international.org/14.0/#sec-ecmascript-language-types-null-type>
    Null,

    /// <https://262.ecma-international.org/14.0/#sec-ecmascript-language-types-boolean-type>
    Boolean(bool),

    /// <https://262.ecma-international.org/14.0/#sec-ecmascript-language-types-string-type>
    String(String),

    Number(Number),

    Symbol,
    BigInt,
    Object,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TypeTag {
    Undefined,
    Null,
    Boolean,
    String,
    Number,
    Symbol,
    BigInt,
    Object,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Number(f64);

impl Number {
    pub const NAN: Self = Self(f64::NAN);
    pub const INFINITY: Self = Self(f64::INFINITY);
    pub const NEG_INFINITY: Self = Self(f64::NEG_INFINITY);
    pub const ZERO: Self = Self(0.);
    pub const NEG_ZERO: Self = Self(-0.);

    #[must_use]
    pub const fn new(value: f64) -> Self {
        Self(value)
    }

    #[must_use]
    pub fn is_nan(&self) -> bool {
        self.0.is_nan()
    }

    /// <https://262.ecma-international.org/14.0/#sec-numeric-types-number-tostring>
    #[must_use]
    pub fn to_string(&self, base: u8) -> String {
        _ = base;
        todo!()
    }

    /// <https://262.ecma-international.org/14.0/#sec-numeric-types-number-add>
    #[must_use]
    pub fn add(&self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreferredType {
    String,
    Number,
}

impl Value {
    #[must_use]
    pub fn type_tag(&self) -> TypeTag {
        match self {
            Self::Undefined => TypeTag::Undefined,
            Self::Null => TypeTag::Null,
            Self::Boolean(_) => TypeTag::Boolean,
            Self::String(_) => TypeTag::String,
            Self::Number(_) => TypeTag::Number,
            Self::Symbol => TypeTag::Symbol,
            Self::Object => TypeTag::Object,
            Self::BigInt => TypeTag::BigInt,
        }
    }

    #[must_use]
    pub const fn is_bigint(&self) -> bool {
        matches!(self, Self::BigInt)
    }

    #[must_use]
    pub const fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    #[must_use]
    pub const fn is_object(&self) -> bool {
        matches!(self, Self::Object)
    }

    /// <https://262.ecma-international.org/14.0/#sec-toprimitive>
    #[must_use]
    pub fn to_primitive(&self, preferred_type: Option<PreferredType>) -> ThrowCompletionOr<Self> {
        // FIXME: Implement 1.
        _ = preferred_type;

        // 2. Return input.
        Ok(self.clone())
    }

    /// <https://262.ecma-international.org/14.0/#sec-tonumeric>
    #[must_use]
    pub fn to_numeric(&self) -> ThrowCompletionOr<Self> {
        // 1. Let primValue be ? ToPrimitive(value, number).
        let prim_value = self.to_primitive(Some(PreferredType::Number))?;

        // 2. If primValue is a BigInt, return primValue.
        if prim_value.is_bigint() {
            return Ok(prim_value);
        }

        // 3. Return ? ToNumber(primValue).
        let number = prim_value.to_number()?;
        Ok(number.into())
    }

    /// <https://262.ecma-international.org/14.0/#sec-tonumber>
    #[must_use]
    pub fn to_number(&self) -> ThrowCompletionOr<Number> {
        match self {
            Self::Number(n) => {
                // 1. If argument is a Number, return argument.
                Ok(*n)
            },
            Self::Symbol | Self::BigInt => {
                // 2. If argument is either a Symbol or a BigInt, throw a TypeError exception.
                Err(Exception::TypeError)
            },
            Self::Undefined => {
                // 3. If argument is undefined, return NaN.
                Ok(Number(f64::NAN))
            },
            Self::Null | Self::Boolean(false) => {
                // 4. If argument is either null or false, return +0𝔽.
                Ok(Number(0.))
            },
            Self::Boolean(true) => {
                // 5. If argument is true, return 1𝔽.
                Ok(Number(1.))
            },
            Self::String(s) => {
                // FIXME: 6. If argument is a String, return StringToNumber(argument).
                _ = s;
                todo!()
            },
            Self::Object => {
                // 7. Assert: argument is an Object.
                //    NOTE: Pointless if we're in this match arm

                // 8. Let primValue be ? ToPrimitive(argument, number).
                let prim_value = self.to_primitive(Some(PreferredType::Number))?;

                // 9. Assert: primValue is not an Object.
                assert!(!prim_value.is_object());

                // 10. Return ? ToNumber(primValue).
                prim_value.to_number()
            },
        }
    }

    /// <https://262.ecma-international.org/14.0/#sec-tostring>
    #[must_use]
    pub fn to_string(self) -> ThrowCompletionOr<String> {
        match self {
            Self::String(s) => {
                // 1. If argument is a String, return argument.
                Ok(s)
            },
            Self::Symbol => {
                // 2. If argument is a Symbol, throw a TypeError exception.
                Err(Exception::TypeError)
            },
            Self::Undefined => {
                // 3. If argument is undefined, return "undefined".
                Ok("undefined".to_string())
            },
            Self::Null => {
                // 4. If argument is null, return "null".
                Ok("null".to_string())
            },
            Self::Boolean(true) => {
                // 5. If argument is true, return "true".
                Ok("true".to_string())
            },
            Self::Boolean(false) => {
                // 6. If argument is false, return "false".
                Ok("false".to_string())
            },
            Self::Number(n) => {
                // 7. If argument is a Number, return Number::toString(argument, 10).
                Ok(n.to_string(10))
            },
            Self::BigInt => {
                // 8. If argument is a BigInt, return BigInt::toString(argument, 10).
                todo!()
            },
            Self::Object => {
                // 9. Assert: argument is an Object.
                //    NOTE: Pointless if we're in this match arm

                // 10. Let primValue be ? ToPrimitive(argument, string).
                let prim_value = self.to_primitive(Some(PreferredType::String))?;

                // 11. Assert: primValue is not an Object.
                assert!(!prim_value.is_object());

                // 12. Return ? ToString(primValue).
                prim_value.to_string()
            },
        }
    }
}

impl From<Number> for Value {
    fn from(value: Number) -> Self {
        Self::Number(value)
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number_add() {
        let any = Number::new(42.);
        assert!(Number::NAN.add(any).is_nan());
        assert!(any.add(Number::NAN).is_nan());

        assert!(Number::NEG_INFINITY.add(Number::INFINITY).is_nan());
        assert!(Number::INFINITY.add(Number::NEG_INFINITY).is_nan());

        assert_eq!(Number::INFINITY.add(any), Number::INFINITY);
        assert_eq!(Number::NEG_INFINITY.add(any), Number::NEG_INFINITY);
        assert_eq!(any.add(Number::INFINITY), Number::INFINITY);
        assert_eq!(any.add(Number::NEG_INFINITY), Number::NEG_INFINITY);

        assert_eq!(Number::NEG_ZERO.add(Number::NEG_ZERO), Number::NEG_ZERO);
    }
}
