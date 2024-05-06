use std::iter::FusedIterator;

/// A value within a comma-separated header
#[derive(Clone, Copy, Debug)]
pub enum HeaderDirective<'a> {
    /// A key-value pair like `max-age=604800`
    KeyValuePair { key: &'a str, value: &'a str },

    /// A plain value like `no-cache`
    Value(&'a str),
}

#[derive(Clone, Debug)]
pub struct CommaSeparatedHeader<'a> {
    is_done: bool,

    remainder: &'a str,
}

impl<'a> CommaSeparatedHeader<'a> {
    #[must_use]
    pub fn new(header_value: &'a str) -> Self {
        Self {
            is_done: false,
            remainder: header_value.trim(),
        }
    }
}

impl<'a> Iterator for CommaSeparatedHeader<'a> {
    type Item = HeaderDirective<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_done {
            return None;
        }

        let Some((chunk, new_remainder)) = self.remainder.split_once(',') else {
            // This is the last chunk
            self.is_done = true;
            return Some(HeaderDirective::from(self.remainder));
        };

        let directive = HeaderDirective::from(chunk.trim_end());
        self.remainder = new_remainder.trim_start();

        Some(directive)
    }
}

impl<'a> FusedIterator for CommaSeparatedHeader<'a> {}

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
