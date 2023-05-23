pub mod color;

/// <https://drafts.csswg.org/css-values-4/#number-value>
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Number {
    Integer(i32),
    Number(f32),
}

impl Number {
    #[must_use]
    pub fn round_to_int(&self) -> i32 {
        match self {
            Self::Integer(i) => *i,
            Self::Number(f) => f.round().clamp(i32::MIN as f32, i32::MAX as f32) as i32,
        }
    }
}
