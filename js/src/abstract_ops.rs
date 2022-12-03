use crate::{
    handle_completion,
    completion::{JSCompletionRecord, JSCompletionRecordType},
    error::TypeError,
    js_type::{JSType, Number, IntegralNumber},
};

pub fn to_primitive(input: JSType) -> JSType {
    match input {
        JSType::Object => todo!("convert object to primitive value"),
        other => other,
    }
}

pub fn to_number(input: JSType) -> JSCompletionRecord<Number> {
    match input {
        JSType::Undefined => JSCompletionRecord::normal(Number::NAN),
        JSType::Null => JSCompletionRecord::normal(Number::ZERO),
        JSType::Boolean(b) => {
            JSCompletionRecord::normal(if b { Number::ONE } else { Number::ZERO })
        }
        JSType::Number(n) => JSCompletionRecord::normal(n),
        JSType::String(s) => JSCompletionRecord::normal(string_to_number(s)),
        JSType::Symbol => JSCompletionRecord::error(Box::new(TypeError)),
        JSType::BigInt => JSCompletionRecord::error(Box::new(TypeError)),
        JSType::Object => todo!(),
    }
}

pub fn to_int32(input: JSType) -> JSCompletionRecord<IntegralNumber> {
    let number = handle_completion!(to_number(input));
    if number.is_nan() || number.is_abs_zero() || !number.is_finite() {
        return JSCompletionRecord::normal(0);
    }
    let int = number.as_float().trunc();
    JSCompletionRecord::normal(int as IntegralNumber)
}

pub fn to_uint32(input: JSType) -> JSCompletionRecord<IntegralNumber> {
    let number = handle_completion!(to_number(input));
    if number.is_nan() || number.is_abs_zero() || !number.is_finite() {
        return JSCompletionRecord::normal(0);
    }
    let int = number.as_float().trunc();
    JSCompletionRecord::normal(int as IntegralNumber)
}

pub fn string_to_number(_s: String) -> Number {
    todo!();
}


