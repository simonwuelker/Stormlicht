//! <https://www.rfc-editor.org/rfc/rfc2616#section-4.2>

mod cache_control;
mod utils;
mod value;

use std::collections::HashMap;

use self::cache_control::CacheControlIterator;

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

    #[must_use]
    pub fn cache_control_directives(&self) -> CacheControlIterator {
        let Some(header) = self.get(Header::CACHE_CONTROL) else {
            return CacheControlIterator::EMPTY;
        };

        CacheControlIterator::new(header)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_each_cache_control_directive() {
        use super::cache_control::CacheControlDirective;

        let mut headers = Headers::default();

        let mut directives = headers.cache_control_directives();
        assert!(directives.next().is_none());

        // This header is nonsense, but valid.
        headers.set(
            Header::CACHE_CONTROL,
            " public, totally-invalid, max-age = 100, no-cache".to_string(),
        );
        let mut directives = headers.cache_control_directives();

        assert_eq!(directives.next(), Some(CacheControlDirective::Public));
        assert_eq!(directives.next(), Some(CacheControlDirective::MaxAge(100)));
        assert_eq!(directives.next(), Some(CacheControlDirective::NoCache));
        assert!(directives.next().is_none());
    }
}
