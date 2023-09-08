pub use std::ascii::Char;

use std::{borrow::Borrow, fmt, ops::Deref, string::String as Utf8String};

use crate::punycode;

#[derive(Clone, Default, PartialEq, Eq)]
pub struct String {
    chars: Vec<Char>,
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
        for b in encoded.bytes() {
            ascii_chars.push(Char::from_u8(b).expect("Punycode encode returned non-ascii byte"))
        }
        Ok(ascii_chars)
    }
}

/// A borrowed [String]
#[repr(transparent)]
#[derive(PartialEq, Eq)]
pub struct Str {
    chars: [Char],
}

impl Str {
    #[must_use]
    pub fn from_ascii_chars(chars: &[Char]) -> &Self {
        // SAFETY: Str is guaranteed to have the same layout as [Char]
        unsafe { &*(chars as *const [Char] as *const Str) }
    }

    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Option<&Self> {
        bytes.as_ascii().map(Self::from_ascii_chars)
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        self.chars.as_str()
    }

    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.chars.as_bytes()
    }

    #[must_use]
    pub fn chars(&self) -> &[Char] {
        &self.chars
    }
}

impl Deref for String {
    type Target = Str;

    fn deref(&self) -> &Self::Target {
        Str::from_ascii_chars(self.chars.as_slice())
    }
}

impl Deref for Str {
    type Target = [Char];

    fn deref(&self) -> &Self::Target {
        &self.chars
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

impl fmt::Debug for Str {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self)
    }
}

impl fmt::Display for Str {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for c in &self.chars {
            write!(f, "{c}")?;
        }
        Ok(())
    }
}

impl Borrow<Str> for String {
    fn borrow(&self) -> &Str {
        self.deref()
    }
}

impl ToOwned for Str {
    type Owned = String;

    fn to_owned(&self) -> Self::Owned {
        String {
            chars: self.chars.to_owned(),
        }
    }
}

impl PartialEq<str> for Str {
    fn eq(&self, other: &str) -> bool {
        self.as_bytes().eq(other.as_bytes())
    }
}

pub trait Write {
    fn write_str(&mut self, s: &Str);

    fn write_char(&mut self, c: Char) {
        self.write_str(Str::from_ascii_chars(&[c]));
    }
}

impl Write for String {
    fn write_str(&mut self, s: &Str) {
        self.push_str(s);
    }

    fn write_char(&mut self, c: Char) {
        self.push(c);
    }
}

impl Write for Utf8String {
    fn write_str(&mut self, s: &Str) {
        self.push_str(s.as_str())
    }

    fn write_char(&mut self, c: Char) {
        self.push(c.as_char())
    }
}
