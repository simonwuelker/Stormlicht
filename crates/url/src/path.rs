use std::mem;

use sl_std::ascii;

pub struct PathSegments<'a> {
    remaining: &'a ascii::Str,
}

impl<'a> PathSegments<'a> {
    #[inline]
    #[must_use]
    pub const fn new(path: &'a ascii::Str) -> Self {
        Self { remaining: path }
    }
}

impl<'a> Iterator for PathSegments<'a> {
    type Item = &'a ascii::Str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.is_empty() {
            return None;
        }

        let start_of_next_component = self.remaining.split_once(ascii::Char::Solidus);

        let Some((component, remaining)) = start_of_next_component else {
            return Some(mem::take(&mut self.remaining));
        };

        self.remaining = remaining;

        Some(component)
    }
}
