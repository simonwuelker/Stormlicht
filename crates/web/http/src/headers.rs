use std::collections::HashMap;

/// Thin wrapper around a [HashMap] to provide case-insensitive
/// key lookup, as is required for HTTP Headers.
#[derive(Clone, Debug, Default)]
pub struct Headers {
    internal: HashMap<String, String>,
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

    pub fn get(&self, header: &str) -> Option<&str> {
        self.internal
            .get(&header.to_ascii_lowercase())
            .map(String::as_str)
    }

    pub fn set(&mut self, header: &str, value: String) {
        // FIXME: We could potentially use a case-insensitive hash algorithm for
        //        the hashmap, which would save us from having to allocate a new string in
        //        str::to_ascii_lowercase
        self.internal.insert(header.to_ascii_lowercase(), value);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.internal
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str()))
    }
}
