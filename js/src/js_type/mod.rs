mod number;

pub use number::{IntegralNumber, Number};

pub enum JSType {
    Undefined,
    Null,
    Boolean(bool),
    String(String),
    Number(Number),
    Symbol,
    BigInt,
    Object,
}
