use super::String;
use std::{ascii::Char, fmt, ops, slice::SliceIndex};

/// A borrowed [String]
#[repr(transparent)]
#[derive(PartialEq, Eq)]
pub struct Str {
    chars: [Char],
}

impl Str {
    #[must_use]
    pub const fn len(&self) -> usize {
        self.chars().len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[must_use]
    pub const fn from_ascii_chars(chars: &[Char]) -> &Self {
        // SAFETY: Str is guaranteed to have the same layout as [Char]
        unsafe { &*(chars as *const [Char] as *const Str) }
    }

    #[must_use]
    pub fn from_ascii_chars_mut(chars: &mut [Char]) -> &mut Self {
        // SAFETY: Str is guaranteed to have the same layout as [Char]
        unsafe { &mut *(chars as *mut [Char] as *mut Str) }
    }

    #[must_use]
    pub const fn from_bytes(bytes: &[u8]) -> Option<&Self> {
        // Cannot use Option::map in a const context
        match bytes.as_ascii() {
            Some(ascii_slice) => Some(Self::from_ascii_chars(ascii_slice)),
            None => None,
        }
    }

    #[must_use]
    pub const fn as_str(&self) -> &str {
        self.chars.as_str()
    }

    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.chars.as_bytes()
    }

    #[must_use]
    pub const fn chars(&self) -> &[Char] {
        &self.chars
    }

    #[must_use]
    pub fn chars_mut(&mut self) -> &mut [Char] {
        &mut self.chars
    }

    #[must_use]
    pub fn rfind(&self, c: Char) -> Option<usize> {
        self.chars
            .iter()
            .enumerate()
            .rev()
            .find(|(_, &element)| element == c)
            .map(|(i, _)| i)
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

macro_rules! slice_index_impl {
    ($for: ty) => {
        unsafe impl SliceIndex<Str> for $for {
            type Output = Str;

            fn get(self, slice: &Str) -> Option<&Self::Output> {
                self.get(slice.chars()).map(Str::from_ascii_chars)
            }

            fn get_mut(self, slice: &mut Str) -> Option<&mut Self::Output> {
                self.get_mut(slice.chars_mut())
                    .map(Str::from_ascii_chars_mut)
            }

            unsafe fn get_unchecked(self, slice: *const Str) -> *const Self::Output {
                self.get_unchecked((*slice).chars()) as *const Self::Output
            }

            unsafe fn get_unchecked_mut(self, slice: *mut Str) -> *mut Self::Output {
                self.get_unchecked_mut((*slice).chars_mut()) as *mut Self::Output
            }

            fn index(self, slice: &Str) -> &Self::Output {
                Str::from_ascii_chars(self.index(slice.chars()))
            }

            fn index_mut(self, slice: &mut Str) -> &mut Self::Output {
                Str::from_ascii_chars_mut(self.index_mut(slice.chars_mut()))
            }
        }
    };
}

slice_index_impl!(ops::Range<usize>);
slice_index_impl!(ops::RangeFrom<usize>);
slice_index_impl!(ops::RangeFull);
slice_index_impl!(ops::RangeInclusive<usize>);
slice_index_impl!(ops::RangeTo<usize>);
slice_index_impl!(ops::RangeToInclusive<usize>);

impl ops::Index<usize> for Str {
    type Output = Char;

    fn index(&self, index: usize) -> &Self::Output {
        &self.chars()[index]
    }
}
impl<T> ops::Index<T> for Str
where
    T: SliceIndex<Str, Output = Str>,
{
    type Output = Self;

    fn index(&self, index: T) -> &Self::Output {
        index.index(self)
    }
}