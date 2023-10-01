pub use std::ascii::Char;

use std::{borrow::Borrow, fmt, mem, ops::Deref, string::String as Utf8String};

use crate::punycode;

use super::Str;

#[derive(Clone, Copy, Debug)]
pub struct NotAscii;

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

    /// Create a ascii String from bytes, if the bytes are valid ASCII
    ///
    /// # Example
    /// ```
    /// # use sl_std::ascii;
    ///
    /// let valid_ascii = b"foo bar".to_vec();
    /// let invalid_ascii = b"foo \xff bar".to_vec();
    ///
    /// assert_eq!(
    ///     ascii::String::from_bytes(valid_ascii)
    ///         .as_deref()
    ///         .map(ascii::Str::as_str),
    ///     Some("foo bar")
    /// );
    /// assert_eq!(ascii::String::from_bytes(invalid_ascii), None);
    /// ```
    pub fn from_bytes(bytes: Vec<u8>) -> Option<Self> {
        if bytes.is_ascii() {
            let chars = unsafe {
                // Ensure the original vector is not dropped.
                let mut wrapped_bytes = mem::ManuallyDrop::new(bytes);

                // SAFETY: Vec<u8> has the same layout as Vec<ascii::Char>
                Vec::from_raw_parts(
                    wrapped_bytes.as_mut_ptr() as *mut Char,
                    wrapped_bytes.len(),
                    wrapped_bytes.capacity(),
                )
            };
            Some(Self { chars })
        } else {
            None
        }
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

impl TryFrom<Utf8String> for String {
    type Error = NotAscii;

    fn try_from(value: Utf8String) -> Result<Self, Self::Error> {
        Self::from_bytes(value.into_bytes()).ok_or(NotAscii)
    }
}

impl PartialEq<Str> for String {
    fn eq(&self, other: &Str) -> bool {
        self.deref().eq(other)
    }
}

impl PartialEq<&str> for String {
    fn eq(&self, other: &&str) -> bool {
        self.as_bytes().eq(other.as_bytes())
    }
}