mod auto;
pub mod color;
mod length;
mod length_percentage;

pub use auto::AutoOr;
pub use length::Length;
pub use length_percentage::LengthPercentage;

/// <https://drafts.csswg.org/css-values-4/#number-value>
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Number {
    Integer(i32),
    Number(f32),
}

impl Number {
    pub fn is_zero(&self) -> bool {
        match self {
            Self::Integer(n) => *n == 0,
            Self::Number(f) => *f == 0.,
        }
    }

    #[must_use]
    pub fn round_to_int(&self) -> i32 {
        match self {
            Self::Integer(i) => *i,
            Self::Number(f) => f.round().clamp(i32::MIN as f32, i32::MAX as f32) as i32,
        }
    }
}

impl From<Number> for f32 {
    fn from(value: Number) -> Self {
        match value {
            Number::Integer(i) => i as f32,
            Number::Number(f) => f,
        }
    }
}
