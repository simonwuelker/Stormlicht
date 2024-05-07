/// A value within a comma-separated header
#[derive(Clone, Copy, Debug)]
pub enum HeaderDirective<'a> {
    /// A key-value pair like `max-age=604800`
    KeyValuePair { key: &'a str, value: &'a str },

    /// A plain value like `no-cache`
    Value(&'a str),
}

#[derive(Clone, Debug)]
pub struct CommaSeparatedHeader {
    is_done: bool,

    offset: usize,

    /// The full header value
    ///
    /// Owned because these headers are usually case-insensitive,
    /// so we call `to_ascii_lowercase` once in the beginning
    value: String,
}

impl CommaSeparatedHeader {
    pub const EMPTY: Self = Self {
        is_done: true,
        offset: 0,
        value: String::new(),
    };

    #[must_use]
    pub fn new(header_value: &str) -> Self {
        Self {
            is_done: false,
            offset: 0,
            value: header_value.to_ascii_lowercase(),
        }
    }

    /// Return the next directive
    ///
    /// This can't be an `Iterator` because the lifetime of the
    /// directive is tied to the lifetime of `self`.
    #[must_use]
    pub fn next_directive<'a>(&'a mut self) -> Option<HeaderDirective<'a>> {
        if self.is_done {
            return None;
        }

        let remainder = &self.value[self.offset..];
        let Some((chunk, _)) = remainder.split_once(',') else {
            // This is the last chunk
            self.is_done = true;
            return Some(HeaderDirective::from(remainder.trim()));
        };

        self.offset += chunk.len() + 1;
        let directive = HeaderDirective::from(chunk.trim());

        Some(directive)
    }
}

impl<'a> From<&'a str> for HeaderDirective<'a> {
    fn from(value: &'a str) -> Self {
        match value.split_once('=') {
            Some((key, value)) => Self::KeyValuePair {
                key: key.trim_end(),
                value: value.trim_start(),
            },
            None => Self::Value(value),
        }
    }
}
