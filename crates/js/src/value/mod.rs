mod number;
pub mod object;
mod reference_record;
mod symbol;

pub use number::Number;
pub use object::Object;
pub use reference_record::{ReferenceRecord, ValueOrReference};
pub use symbol::Symbol;

use crate::{
    bytecode::{Exception, ThrowCompletionOr},
    parser::Identifier,
};

const SPEC_CANNOT_FAIL: &str =
    "This operation cannot fail according to the specification (indicated by '!')";

/// Defines the order in which operations should be applied to operands
///
/// See for example <https://262.ecma-international.org/14.0/#sec-islessthan>
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LeftFirst {
    Yes,
    No,
}

/// <https://262.ecma-international.org/14.0/#sec-ecmascript-language-types>
#[derive(Clone, Debug, Default, PartialEq)]
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

    Symbol(Symbol),
    BigInt,
    Object(Object),
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreferredType {
    String,
    Number,
}

impl Value {
    /// Assume the value to be an object and return a reference to it
    ///
    /// ## Panics
    /// Panics if the value is not an object.
    #[must_use]
    pub fn as_object(&self) -> &Object {
        match self {
            Self::Object(o) => &o,
            _ => unreachable!("Value is not an object"),
        }
    }

    #[must_use]
    pub fn type_tag(&self) -> TypeTag {
        match self {
            Self::Undefined => TypeTag::Undefined,
            Self::Null => TypeTag::Null,
            Self::Boolean(_) => TypeTag::Boolean,
            Self::String(_) => TypeTag::String,
            Self::Number(_) => TypeTag::Number,
            Self::Symbol(_) => TypeTag::Symbol,
            Self::Object(_) => TypeTag::Object,
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
        matches!(self, Self::Object(_))
    }

    #[must_use]
    pub const fn is_undefined(&self) -> bool {
        matches!(self, Self::Undefined)
    }

    #[must_use]
    pub fn as_string(&self) -> Option<&str> {
        if let Self::String(s) = self {
            Some(s.as_str())
        } else {
            None
        }
    }

    /// <https://262.ecma-international.org/#sec-isstrictlyequal>
    pub fn is_strictly_equal(x: &Self, y: &Self) -> ThrowCompletionOr<bool> {
        // 1. If Type(x) is not Type(y), return false.
        if x.type_tag() != y.type_tag() {
            return Ok(false);
        }

        // 2. If x is a Number, then
        if let (Value::Number(x), Value::Number(y)) = (x, y) {
            // a. Return Number::equal(x, y).
            return Ok(Number::equal(*x, *y));
        }

        // 3. Return SameValueNonNumber(x, y).
        Ok(Self::same_value_non_number(x, y))
    }

    /// <https://262.ecma-international.org/14.0/#sec-islessthan>
    pub fn is_less_than(x: &Self, y: &Self, left_first: LeftFirst) -> ThrowCompletionOr<Value> {
        // 1. If LeftFirst is true, then
        let px;
        let py;
        if left_first == LeftFirst::Yes {
            // a. Let px be ? ToPrimitive(x, number).
            px = x.to_primitive(Some(PreferredType::Number))?;

            // b. Let py be ? ToPrimitive(y, number).
            py = y.to_primitive(Some(PreferredType::Number))?;
        }
        // 2. Else,
        else {
            // a. NOTE: The order of evaluation needs to be reversed to preserve left to right evaluation.
            // b. Let py be ? ToPrimitive(y, number).
            py = y.to_primitive(Some(PreferredType::Number))?;

            // c. Let px be ? ToPrimitive(x, number).
            px = x.to_primitive(Some(PreferredType::Number))?;
        }

        // 3. If px is a String and py is a String, then
        if let (Some(px), Some(py)) = (px.as_string(), py.as_string()) {
            // a. Let lx be the length of px.
            let lx = px.len();

            // b. Let ly be the length of py.
            let ly = py.len();

            // c. For each integer i such that 0 ≤ i < min(lx, ly), in ascending order, do
            //    i. Let cx be the numeric value of the code unit at index i within px.
            //    ii. Let cy be the numeric value of the code unit at index i within py.
            for (cx, cy) in px.chars().zip(py.chars()) {
                // iii. If cx < cy, return true.
                if cx < cy {
                    return Ok(true.into());
                }

                // iv. If cx > cy, return false.
                if cx > cy {
                    return Ok(false.into());
                }
            }

            // d. If lx < ly, return true. Otherwise, return false.
            return Ok((lx < ly).into());
        }
        // 4. Else,
        else {
            // a. FIXME: If px is a BigInt and py is a String, then
            // b. FIXME: If px is a String and py is a BigInt, then
            // c. NOTE: Because px and py are primitive values, evaluation order is not important.
            // d. Let nx be ? ToNumeric(px).
            let nx = px.to_numeric()?;

            // e. Let ny be ? ToNumeric(py).
            let ny = py.to_numeric()?;

            // f. If Type(nx) is Type(ny), then
            match (nx, ny) {
                // i. If nx is a Number, then
                (Self::Number(nx), Self::Number(ny)) => {
                    // 1. Return Number::lessThan(nx, ny).
                    return Ok(Number::less_than(nx, ny));
                },
                (Self::BigInt, Self::BigInt) => {
                    todo!();
                },
                _ => {
                    todo!("mixed bigint/number comparison")
                },
            }
        }
    }

    /// <https://262.ecma-international.org/#sec-samevaluenonnumber>
    fn same_value_non_number(x: &Self, y: &Self) -> bool {
        // 1. Assert: Type(x) is Type(y).
        assert_eq!(x.type_tag(), y.type_tag());

        match (x, y) {
            (Self::Null, Self::Null) | (Self::Undefined, Self::Undefined) => {
                // 2. If x is either null or undefined, return true.
                true
            },
            (Self::BigInt, Self::BigInt) => {
                // 3. If x is a BigInt, then
                todo!()
            },
            (Self::String(x), Self::String(y)) => {
                // 4. If x is a String, then
                //    a. If x and y have the same length and the same code units in the same positions, return true; otherwise, return false.
                return x == y;
            },
            (Self::Boolean(x), Self::Boolean(y)) => {
                // 5. If x is a Boolean, then
                //    a. If x and y are both true or both false, return true; otherwise, return false.
                return x == y;
            },
            _ => {
                // 6. NOTE: All other ECMAScript language values are compared by identity.
                // 7. If x is y, return true; otherwise, return false.
                todo!()
            },
        }
    }

    /// <https://262.ecma-international.org/#sec-islooselyequal>
    pub fn is_loosely_equal(x: &Self, y: &Self) -> ThrowCompletionOr<bool> {
        // 1. 1. 1. If Type(x) is Type(y), then
        if x.type_tag() == y.type_tag() {
            // a. Return IsStrictlyEqual(x, y).
            return Self::is_strictly_equal(x, y);
        }

        match (x.type_tag(), y.type_tag()) {
            (TypeTag::Null, TypeTag::Undefined) => {
                // 2. If x is null and y is undefined, return true.
                return Ok(true);
            },
            (TypeTag::Undefined, TypeTag::Null) => {
                // 3. If x is undefined and y is null, return true.
                return Ok(true);
            },
            (TypeTag::Number, TypeTag::String) => {
                // 5. If x is a Number and y is a String, return ! IsLooselyEqual(x, ! ToNumber(y)).
                let y_numeric = y.to_number().expect(SPEC_CANNOT_FAIL).into();
                let is_equal = Self::is_loosely_equal(x, &y_numeric).expect(SPEC_CANNOT_FAIL);
                return Ok(is_equal);
            },
            (TypeTag::String, TypeTag::Number) => {
                // 6. If x is a String and y is a Number, return ! IsLooselyEqual(! ToNumber(x), y).
                let x_numeric = x.to_number().expect(SPEC_CANNOT_FAIL).into();
                let is_equal = Self::is_loosely_equal(&x_numeric, y).expect(SPEC_CANNOT_FAIL);
                return Ok(is_equal);
            },
            (TypeTag::BigInt, TypeTag::String) => {
                // 7. If x is a BigInt and y is a String, then
                todo!()
            },
            (TypeTag::String, TypeTag::BigInt) => {
                // 8. If x is a String and y is a BigInt, return ! IsLooselyEqual(y, x).
                let is_equal = Self::is_loosely_equal(y, x).expect(SPEC_CANNOT_FAIL);
                Ok(is_equal)
            },
            (TypeTag::Boolean, _) => {
                // 9. If x is a Boolean, return ! IsLooselyEqual(! ToNumber(x), y).
                let x_number = x.to_number().expect(SPEC_CANNOT_FAIL).into();
                let is_equal = Self::is_loosely_equal(&x_number, y).expect(SPEC_CANNOT_FAIL);
                Ok(is_equal)
            },
            (_, TypeTag::Boolean) => {
                // 10. If y is a Boolean, return ! IsLooselyEqual(x, ! ToNumber(y)).
                let y_number = y.to_number().expect(SPEC_CANNOT_FAIL).into();
                let is_equal = Self::is_loosely_equal(x, &y_number).expect(SPEC_CANNOT_FAIL);
                Ok(is_equal)
            },
            (
                TypeTag::String | TypeTag::Number | TypeTag::BigInt | TypeTag::Symbol,
                TypeTag::Object,
            ) => {
                // 11. If x is either a String, a Number, a BigInt, or a Symbol and y is an Object, return ! IsLooselyEqual(x, ? ToPrimitive(y)).
                todo!()
            },
            (
                TypeTag::Object,
                TypeTag::String | TypeTag::Number | TypeTag::BigInt | TypeTag::Symbol,
            ) => {
                // 12. If x is an Object and y is either a String, a Number, a BigInt, or a Symbol, return ! IsLooselyEqual(? ToPrimitive(x), y).
                todo!()
            },
            (TypeTag::BigInt, TypeTag::Number) | (TypeTag::Number, TypeTag::BigInt) => {
                // 13. If x is a BigInt and y is a Number, or if x is a Number and y is a BigInt, then
                todo!()
            },
            _ => {
                // 14. Return false.
                Ok(false)
            },
        }
    }

    /// <https://262.ecma-international.org/14.0/#sec-applystringornumericbinaryoperator>
    pub fn apply_string_or_numeric_binary_operator(
        mut lval: Self,
        op: StringOrNumericBinaryOperator,
        mut rval: Self,
    ) -> Result<Self, Exception> {
        // 1. If opText is +, then
        if op == StringOrNumericBinaryOperator::Add {
            // a. Let lprim be ? ToPrimitive(lval).
            let lprim = lval.to_primitive(None)?;

            // b. Let rprim be ? ToPrimitive(rval).
            let rprim = rval.to_primitive(None)?;

            // c. If lprim is a String or rprim is a String, then
            if lprim.is_string() || rprim.is_string() {
                // i. Let lstr be ? ToString(lprim).
                let lstr = lprim.to_string()?;

                // ii. Let rstr be ? ToString(rprim).
                let rstr = rprim.to_string()?;

                // iii. Return the string-concatenation of lstr and rstr.
                return Ok(format!("{lstr}{rstr}").into());
            }

            // d. Set lval to lprim.
            lval = lprim;

            // e. Set rval to rprim.
            rval = rprim;
        }

        // 2. NOTE: At this point, it must be a numeric operation.
        // 3. Let lnum be ? ToNumeric(lval).
        let lnum = lval.to_numeric()?;

        // 4. Let rnum be ? ToNumeric(rval).
        let rnum = rval.to_numeric()?;

        // 5. If Type(lnum) is not Type(rnum), throw a TypeError exception.
        if lval.type_tag() != rval.type_tag() {
            return Err(Exception::type_error());
        }

        // 6. FIXME: If lnum is a BigInt, then

        // 7. Let operation be the abstract operation associated with opText and Type(lnum) in the following table:
        // 8. Return operation(lnum, rnum).
        let result = match (lnum, rnum) {
            (Value::Number(lhs), Value::Number(rhs)) => match op {
                StringOrNumericBinaryOperator::Add => lhs.add(rhs).into(),
                StringOrNumericBinaryOperator::Subtract => lhs.subtract(rhs).into(),
                StringOrNumericBinaryOperator::Multiply => lhs.multiply(rhs).into(),
                StringOrNumericBinaryOperator::Divide => lhs.divide(rhs).into(),
                StringOrNumericBinaryOperator::Modulo => lhs.remainder(rhs).into(),
                StringOrNumericBinaryOperator::ShiftLeft => lhs.shift_left(rhs).into(),
                StringOrNumericBinaryOperator::ShiftRightSigned => {
                    lhs.shift_right_signed(rhs).into()
                },
                StringOrNumericBinaryOperator::ShiftRightUnsigned => {
                    lhs.shift_right_unsigned(rhs).into()
                },
                StringOrNumericBinaryOperator::Exponentiate => lhs.exponentiate(rhs).into(),
                StringOrNumericBinaryOperator::BitwiseAnd => lhs.bitwise_and(rhs).into(),
                StringOrNumericBinaryOperator::BitwiseOr => lhs.bitwise_or(rhs).into(),
                StringOrNumericBinaryOperator::BitwiseExclusiveOr => lhs.bitwise_xor(rhs).into(),
            },
            (Value::BigInt, Value::BigInt) => todo!(),
            _ => unreachable!(),
        };

        Ok(result)
    }

    /// <https://262.ecma-international.org/14.0/#sec-toobject>
    pub fn to_object(&self) -> ThrowCompletionOr<Object> {
        match self {
            Self::Undefined | Self::Null => {
                // Throw a TypeError exception.
                Err(Exception::type_error())
            },
            Self::Object(o) => {
                // Return argument.
                Ok(o.clone())
            },
            _ => {
                // FIXME: Implement
                Err(Exception::type_error())
            },
        }
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
            Self::Symbol(_) | Self::BigInt => {
                // 2. If argument is either a Symbol or a BigInt, throw a TypeError exception.
                Err(Exception::type_error())
            },
            Self::Undefined => {
                // 3. If argument is undefined, return NaN.
                Ok(Number::NAN)
            },
            Self::Null | Self::Boolean(false) => {
                // 4. If argument is either null or false, return +0𝔽.
                Ok(Number::ZERO)
            },
            Self::Boolean(true) => {
                // 5. If argument is true, return 1𝔽.
                Ok(Number::ONE)
            },
            Self::String(s) => {
                // FIXME: 6. If argument is a String, return StringToNumber(argument).
                _ = s;
                todo!()
            },
            Self::Object(_) => {
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

    /// <https://262.ecma-international.org/14.0/#sec-toboolean>
    #[must_use]
    pub fn to_boolean(&self) -> bool {
        match self {
            Self::Boolean(b) => {
                // 1. If argument is a Boolean, return argument.
                *b
            },
            Self::Undefined | Self::Null => {
                // 2. If argument is one of undefined, null, +0𝔽, -0𝔽, NaN, 0ℤ, or the empty String, return false.
                false
            },
            Self::Number(n)
                if n == &Number::NEG_ZERO || n == &Number::ZERO || n == &Number::NAN =>
            {
                // 2. If argument is one of undefined, null, +0𝔽, -0𝔽, NaN, 0ℤ, or the empty String, return false.
                false
            },
            Self::String(s) if s.is_empty() => {
                // 2. If argument is one of undefined, null, +0𝔽, -0𝔽, NaN, 0ℤ, or the empty String, return false.
                false
            },
            _ => {
                // 3. NOTE: This step is replaced in section B.3.6.1.
                //          (https://262.ecma-international.org/14.0/#sec-IsHTMLDDA-internal-slot-to-boolean)

                // 4. Return true.
                true
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
            Self::Symbol(_) => {
                // 2. If argument is a Symbol, throw a TypeError exception.
                Err(Exception::type_error())
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
            Self::Object(_) => {
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

    /// <https://262.ecma-international.org/14.0/#sec-evaluate-property-access-with-identifier-key>
    #[must_use]
    pub fn evaluate_property_access_with_identifier_key(
        base_value: Self,
        identifier: Identifier,
    ) -> ReferenceRecord {
        // 1. Let propertyNameString be StringValue of identifierName.
        let property_name_string = identifier.to_string();

        // 2. Return the Reference Record { [[Base]]: baseValue, [[ReferencedName]]: propertyNameString,
        //    [[Strict]]: strict, [[ThisValue]]: empty }.
        ReferenceRecord {
            base: base_value,
            referenced_name: property_name_string,
        }
    }
}

/// <https://262.ecma-international.org/14.0/#sec-evaluatestringornumericbinaryexpression>
pub fn evaluate_string_or_numeric_binary_expression(
    left_operand: ValueOrReference,
    op: StringOrNumericBinaryOperator,
    right_operand: ValueOrReference,
) -> ThrowCompletionOr<Value> {
    // 1. Let lref be ? Evaluation of leftOperand.
    // 2. Let lval be ? GetValue(lref).
    let lval = left_operand.get_value()?;

    // 3. Let rref be ? Evaluation of rightOperand.
    // 4. Let rval be ? GetValue(rref).
    let rval = right_operand.get_value()?;

    // 5. Return ? ApplyStringOrNumericBinaryOperator(lval, opText, rval).
    Value::apply_string_or_numeric_binary_operator(lval, op, rval)
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

impl From<Object> for Value {
    fn from(value: Object) -> Self {
        Self::Object(value)
    }
}

/// `opText` in <https://262.ecma-international.org/14.0/#sec-applystringornumericbinaryoperator>
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StringOrNumericBinaryOperator {
    Exponentiate,
    Multiply,
    Divide,
    Modulo,
    Add,
    Subtract,
    ShiftLeft,
    ShiftRightSigned,
    ShiftRightUnsigned,
    BitwiseAnd,
    BitwiseExclusiveOr,
    BitwiseOr,
}
