//! <https://www.rfc-editor.org/rfc/rfc2616#section-4.2>

mod utils;
mod value;

use std::collections::HashMap;

pub use value::Header;

/// Thin wrapper around a [HashMap] to provide case-insensitive
/// key lookup, as is required for HTTP Headers.
#[derive(Clone, Debug, Default)]
pub struct Headers {
    internal: HashMap<Header, String>,
}

impl Headers {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            internal: HashMap::with_capacity(capacity),
        }
    }

    pub fn clear(&mut self) {
        self.internal.clear()
    }

    pub fn get(&self, header: Header) -> Option<&str> {
        self.internal.get(&header).map(String::as_str)
    }

    pub fn set(&mut self, header: Header, value: String) {
        self.internal.insert(header, value);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Header, &str)> {
        self.internal
            .iter()
            .map(|(key, value)| (key, value.as_str()))
    }
}
