use crate::Value;

/// <https://262.ecma-international.org/14.0/#sec-completion-record-specification-type>
pub type ThrowCompletionOr<T> = Result<T, Exception>;

#[derive(Clone, Debug)]
pub struct Exception {
    value: Value,
}

impl Exception {
    #[must_use]
    pub const fn new(value: Value) -> Self {
        Self { value }
    }

    #[must_use]
    pub fn type_error() -> Self {
        // FIXME: This should be a "typeerror" object, but we don't
        //        really support objects yet
        Self {
            value: "TypeError".to_string().into(),
        }
    }
}
