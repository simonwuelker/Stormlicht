//! The *Completion Record* specification type is used to explain the runtime propagation of values
//! and control flow such as the behaviour of statements (**break**, **continue**, **return** and **throw**) that
//! perform nonlocal transfers of control.

use crate::error::JSError;

pub struct JSCompletionRecord<T> {
    /// The type of completion that occurred
    pub completion_type: JSCompletionRecordType<T>,
    ///  The target label for directed control transfers.
    pub target: Option<String>,
}

impl<T> JSCompletionRecord<T> {
    pub fn normal(value: T) -> Self {
        Self {
            completion_type: JSCompletionRecordType::Normal(value),
            target: None,
        }
    }

    pub fn error(exception: Box<dyn JSError>) -> Self {
        Self {
            completion_type: JSCompletionRecordType::Throw(exception),
            target: None,
        }
    }

    pub fn unwrap(self) -> T {
        match self.completion_type {
            JSCompletionRecordType::Normal(value) => value,
            _ => panic!("Unexpected JSCompletionRecordType"),
        }
    }
}

pub enum JSCompletionRecordType<T> {
    /// Normal completion, contains
    /// the value that was produced.
    Normal(T),
    /// Break completion
    Break,
    /// Continue completion
    Continue,
    /// Return completion
    Return,
    /// Throw completion
    Throw(Box<dyn JSError>),
}

impl<T> JSCompletionRecordType<T> {
    pub fn is_normal(&self) -> bool {
        match self {
            Self::Normal(_) => true,
            _ => false,
        }
    }

    pub fn is_abrupt(&self) -> bool {
        !self.is_normal()
    }
}

#[macro_export]
macro_rules! handle_completion {
    ($i: expr) => {
        match ($i).completion_type {
            JSCompletionRecordType::Normal(value) => value,
            JSCompletionRecordType::Throw(error) => return JSCompletionRecord::error(error),
            _ => todo!(),
        }
    }
}
