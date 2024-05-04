use std::{ascii::Char, borrow::Borrow, fmt, mem, ops::Deref, string::String as Utf8String};

use crate::punycode;

use super::Str;

#[derive(Clone, Copy, Debug)]
pub struct NotAscii;

/// A String that is guaranteed to only contain ASCII data.
///
/// A [ascii::String](String) owns its data, for the borrowed version see [ascii::Str](Str).
///
/// The intention is that this can be used as a drop-in replacement for [std::string::String]
/// in cases where only ascii data is required. Many public API functions are the
/// same as in the standard library ([new](String::new), [with_capacity](String::with_capacity), [len](Str::len) etc).
///
/// Don't import this directly, instead import it's parent module and
/// use it as `ascii::String`.
///
/// # Example
/// ```
/// # use sl_std::ascii;
///
/// // Valid ascii data
/// let ascii = "Hello World";
/// let ascii_string = ascii::String::try_from(ascii);
/// assert!(ascii_string.is_ok());
///
/// let unicode = "💖💖💖💖💖";
/// let unicode_as_ascii = ascii::String::try_from(unicode);
/// assert!(unicode_as_ascii.is_err());
///
/// // You can use it like the standard library string:
/// let mut foo = ascii::String::with_capacity(10);
/// foo.push_str("abcde".try_into().unwrap());
/// assert_eq!(foo.len(), 5);
/// ```
#[cfg_attr(
    feature = "serialize",
    derive(serialize::Serialize, serialize::Deserialize)
)]
#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct String {
    pub(super) chars: Vec<Char>,
}

impl String {
    /// Creates a new empty `String`.
    ///
    /// Given that the `String` is empty, this will not allocate any initial buffer.
    /// While that means that this initial operation is very inexpensive,
    /// it may cause excessive allocation later when you add data.
    /// If you have an idea of how much data the String will hold, consider the
    /// [with_capacity](String::with_capacity) method to prevent excessive re-allocation.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use sl_std::ascii;
    /// let s = ascii::String::new();
    /// ```
    pub fn new() -> Self {
        Self { chars: Vec::new() }
    }

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
            // Ensure the original vector is not dropped.
            let mut wrapped_bytes = mem::ManuallyDrop::new(bytes);

            // SAFETY: Vec<u8> has the same layout as Vec<ascii::Char>
            let chars = unsafe {
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

    #[inline]
    #[must_use]
    pub fn as_ascii_str(&self) -> &'_ Str {
        self.deref()
    }

    pub fn into_bytes(self) -> Vec<u8> {
        // Ensure the original vector is not dropped.
        let mut wrapped_chars = mem::ManuallyDrop::new(self.chars);

        // SAFETY: Vec<u8> has the same layout as Vec<ascii::Char>
        unsafe {
            Vec::from_raw_parts(
                wrapped_chars.as_mut_ptr() as *mut u8,
                wrapped_chars.len(),
                wrapped_chars.capacity(),
            )
        }
    }

    #[must_use]
    #[inline]
    pub const fn from_chars(chars: Vec<Char>) -> Self {
        Self { chars }
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

impl<'a> FromIterator<&'a Str> for String {
    fn from_iter<T: IntoIterator<Item = &'a Str>>(iter: T) -> Self {
        let mut result = Self::new();

        for item in iter {
            result.push_str(item);
        }

        result
    }
}

impl TryFrom<&str> for String {
    type Error = NotAscii;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Str::from_bytes(value.as_bytes())
            .map(Str::to_owned)
            .ok_or(NotAscii)
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

impl From<String> for Utf8String {
    fn from(value: String) -> Self {
        let ascii_bytes = value.into_bytes();
        debug_assert!(ascii_bytes.is_ascii()); // Chance at catching some UB

        // SAFETY: Ascii is guaranteed to be a subset of valid utf8
        unsafe { Utf8String::from_utf8_unchecked(ascii_bytes) }
    }
}
