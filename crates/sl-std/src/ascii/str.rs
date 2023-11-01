use super::{AsciiCharExt, NotAscii, ReverseSearcher, Searcher, String};
use std::{ascii::Char, fmt, iter::FusedIterator, ops, slice::SliceIndex};

/// A borrowed [String]
#[repr(transparent)]
#[derive(PartialEq, Eq)]
pub struct Str {
    chars: [Char],
}

impl Str {
    pub const EMPTY: &'static Self = Self::from_ascii_chars(&[]);

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

    #[inline]
    #[must_use]
    pub const fn as_str(&self) -> &str {
        self.chars.as_str()
    }

    #[inline]
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.chars.as_bytes()
    }

    #[inline]
    #[must_use]
    pub const fn chars(&self) -> &[Char] {
        &self.chars
    }

    #[inline]
    #[must_use]
    pub fn chars_mut(&mut self) -> &mut [Char] {
        &mut self.chars
    }

    pub fn lines(&self) -> SplitIterator<'_, for<'a> fn(&'a std::ascii::Char) -> bool> {
        self.split(Char::is_newline)
    }

    /// Split the string at the occurences of a pattern
    ///
    /// The matched substring will not be contained in any of the segments.
    ///
    /// # Examples
    /// Basic Usage:
    /// ```
    /// # use sl_std::ascii;
    /// let haystack: &ascii::Str = "Lorem ipsum dolor".try_into().unwrap();
    /// let mut splits = haystack.split("m ");
    /// assert_eq!(splits.next().map(ascii::Str::as_str), Some("Lore"));
    /// assert_eq!(splits.next().map(ascii::Str::as_str), Some("ipsu"));
    /// assert_eq!(splits.next().map(ascii::Str::as_str), Some("dolor"));
    /// assert!(splits.next().is_none())
    /// ````
    pub fn split<'a, P: super::Pattern<'a>>(&'a self, pattern: P) -> SplitIterator<'a, P> {
        SplitIterator {
            is_done: false,
            start: 0,
            end: self.len(),
            searcher: pattern.into_searcher(self),
        }
    }

    /// Find a pattern in the string
    ///
    /// # Examples
    /// ```
    /// #![feature(ascii_char_variants, ascii_char)]
    /// # use sl_std::ascii;
    ///
    /// let haystack: &ascii::Str = "abcdef".try_into().unwrap();
    ///
    /// assert_eq!(haystack.find(ascii::Char::SmallB), Some(1));
    /// assert_eq!(haystack.find(ascii::Char::SmallX), None)
    /// ```
    #[must_use]
    pub fn find<'a, P: super::Pattern<'a>>(&'a self, pattern: P) -> Option<usize> {
        pattern
            .into_searcher(self)
            .next_match()
            .map(|(start, _end)| start)
    }

    /// Replace occurences of a pattern with another string
    ///
    /// # Examples
    /// Basic Usage:
    /// ```
    /// # use sl_std::ascii;
    /// let haystack: &ascii::Str = "this is old".try_into().unwrap();
    ///
    /// assert_eq!("this is new", haystack.replace("old", "new".try_into().unwrap()).as_str());
    /// assert_eq!("than an old", haystack.replace("is", "an".try_into().unwrap()).as_str());
    /// ````
    pub fn replace<'a, P: super::Pattern<'a>>(&'a self, pattern: P, replace_with: &Self) -> String {
        let mut result = String::new();

        let mut last_match_end = 0;
        for (start, end) in self.match_indices(pattern) {
            result.push_str(&self[last_match_end..start]);
            result.push_str(replace_with);
            last_match_end = end;
        }
        result.push_str(&self[last_match_end..]);
        result
    }

    pub fn match_indices<'a, P: super::Pattern<'a>>(
        &'a self,
        pattern: P,
    ) -> AsciiMatchIndices<'a, P> {
        AsciiMatchIndices {
            searcher: pattern.into_searcher(self),
        }
    }

    /// Find a pattern from the right in the string
    ///
    /// Note that the index returned will still be the start of the pattern
    /// (the index of the *left*most character)
    ///
    /// # Examples
    /// ```
    /// #![feature(ascii_char_variants, ascii_char)]
    /// # use sl_std::ascii;
    ///
    /// let haystack: &ascii::Str = "abcdef".try_into().unwrap();
    ///
    /// assert_eq!(haystack.rfind(ascii::Char::SmallE), Some(4));
    /// assert_eq!(haystack.rfind(ascii::Char::SmallX), None)
    /// ```
    #[must_use]
    pub fn rfind<'a, P: super::Pattern<'a>>(&'a self, pattern: P) -> Option<usize>
    where
        P::Searcher: ReverseSearcher<'a>,
    {
        pattern
            .into_searcher(self)
            .next_match_back()
            .map(|(start, _end)| start)
    }

    #[inline]
    #[must_use]
    pub fn split_once(&self, split_at: Char) -> Option<(&Self, &Self)> {
        let split_index = self.find(split_at)?;
        let parts = (&self[..split_index], &self[split_index + 1..]);
        Some(parts)
    }

    #[inline]
    #[must_use]
    pub fn split_at(&self, index: usize) -> (&Self, &Self) {
        (&self[..index], &self[index..])
    }

    #[inline]
    #[must_use]
    pub fn trim_end(&self, trim: Char) -> &Self {
        let num_chars_to_remove = self
            .chars()
            .iter()
            .rev()
            .position(|&c| c != trim)
            .unwrap_or(self.len());
        &self[..self.len() - num_chars_to_remove]
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

impl<'a> TryFrom<&'a [u8]> for &'a Str {
    type Error = NotAscii;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        Str::from_bytes(value).ok_or(NotAscii)
    }
}

impl<'a> TryFrom<&'a str> for &'a Str {
    type Error = NotAscii;

    fn try_from(value: &'a str) -> Result<Self, NotAscii> {
        Str::from_bytes(value.as_bytes()).ok_or(NotAscii)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AsciiMatchIndices<'a, P>
where
    P: super::Pattern<'a>,
{
    searcher: P::Searcher,
}

impl<'a, P> Iterator for AsciiMatchIndices<'a, P>
where
    P: super::Pattern<'a>,
{
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        self.searcher.next_match()
    }
}

impl<'a, P> FusedIterator for AsciiMatchIndices<'a, P> where P: super::Pattern<'a> {}

#[derive(Clone, Copy, Debug)]
pub struct SplitIterator<'a, P>
where
    P: super::Pattern<'a>,
{
    is_done: bool,
    start: usize,
    end: usize,
    searcher: P::Searcher,
}

impl<'a, P> Iterator for SplitIterator<'a, P>
where
    P: super::Pattern<'a>,
{
    type Item = &'a Str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_done {
            return None;
        }

        let haystack = self.searcher.haystack();
        match self.searcher.next_match() {
            None => {
                self.is_done = true;
                Some(&haystack[self.start..self.end])
            },
            Some((match_start, match_end)) => {
                let split_item = &haystack[self.start..match_start];
                self.start = match_end;
                Some(split_item)
            },
        }
    }
}

impl<'a, P> DoubleEndedIterator for SplitIterator<'a, P>
where
    P: super::Pattern<'a>,
    P::Searcher: ReverseSearcher<'a>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.is_done {
            return None;
        }

        let haystack = self.searcher.haystack();
        match self.searcher.next_match_back() {
            None => {
                self.is_done = true;
                Some(&haystack[self.start..self.end])
            },
            Some((match_start, match_end)) => {
                let split_item = &haystack[match_end..self.end];
                self.end = match_start;
                Some(split_item)
            },
        }
    }
}

impl<'a, P> FusedIterator for SplitIterator<'a, P> where P: super::Pattern<'a> {}
