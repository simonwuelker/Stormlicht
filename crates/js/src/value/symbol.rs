//! <https://262.ecma-international.org/14.0/#sec-ecmascript-language-types-symbol-type>

use std::sync::atomic::{AtomicUsize, Ordering};

const EPOCH: AtomicUsize = AtomicUsize::new(0);

/// <https://262.ecma-international.org/14.0/#sec-ecmascript-language-types-symbol-type>
#[derive(Clone, Debug)]
pub struct Symbol {
    epoch: usize,
    description: Option<String>,
}

impl Symbol {
    #[must_use]
    pub fn new(description: Option<String>) -> Self {
        Self {
            epoch: EPOCH.fetch_add(1, Ordering::Relaxed),
            description,
        }
    }

    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.description.as_ref().map(String::as_str)
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        self.epoch == other.epoch
    }
}

impl Eq for Symbol {}
