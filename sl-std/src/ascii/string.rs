pub use std::ascii::Char;

use std::{borrow::Borrow, fmt, ops::Deref};

use crate::punycode;

use super::Str;

#[derive(Clone, Default, PartialEq, Eq)]
pub struct String {
    pub(super) chars: Vec<Char>,
}

impl String {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            chars: Vec::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, c: Char) {
        self.chars.push(c);
    }

    pub fn push_str(&mut self, s: &Str) {
        self.chars.extend_from_slice(s.chars())
    }

    pub fn clear(&mut self) {
        self.chars.clear()
    }

    pub fn from_utf8_punycode(utf8_string: &str) -> Result<Self, punycode::PunyCodeError> {
        let encoded = punycode::punycode_encode(utf8_string)?;

        // FIXME: This second iteration isn't strictly necessary, since
        // punycode_encode is never going to return anything other than ascii.
        // Get rid of it, preferrably without using unsafe code
        let mut ascii_chars = Self::with_capacity(encoded.len());
        for &b in encoded.as_bytes() {
            ascii_chars.push(Char::from_u8(b).expect("Punycode encode returned non-ascii byte"))
        }
        Ok(ascii_chars)
    }
}

impl Deref for String {
    type Target = Str;

    fn deref(&self) -> &Self::Target {
        Str::from_ascii_chars(self.chars.as_slice())
    }
}

impl<T> AsRef<T> for String
where
    T: ?Sized,
    <String as Deref>::Target: AsRef<T>,
{
    fn as_ref(&self) -> &T {
        self.deref().as_ref()
    }
}

impl fmt::Debug for String {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.deref().fmt(f)
    }
}

impl fmt::Display for String {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.deref().fmt(f)
    }
}

impl Borrow<Str> for String {
    fn borrow(&self) -> &Str {
        self.deref()
    }
}

impl FromIterator<Char> for String {
    fn from_iter<T: IntoIterator<Item = Char>>(iter: T) -> Self {
        Self {
            chars: iter.into_iter().collect(),
        }
    }
}
