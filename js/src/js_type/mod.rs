mod number;

pub use number::{Number, IntegralNumber};

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
