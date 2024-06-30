use super::{AsciiCharExt, NotAscii, Pattern, ReverseSearcher, Searcher, String};
use std::{ascii::Char, fmt, iter::FusedIterator, ops, slice::SliceIndex};

/// A borrowed [String]
#[repr(transparent)]
#[cfg_attr(feature = "serialize", derive(serialize::Serialize))]
#[derive(PartialEq, Eq, Hash)]
pub struct Str {
    chars: [Char],
}

impl Str {
    /// The empty [Str]
    pub const EMPTY: &'static Self = Self::from_ascii_chars(&[]);

    /// Returns the length of `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(ascii_char_variants, ascii_char)]
    /// # use sl_std::ascii;
    ///
    /// let foo: &ascii::Str = "foo".try_into().unwrap();
    /// assert_eq!(foo.len(), 3);
    /// ```
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.chars().len()
    }

    /// Returns `true` if `self` has a length of zero bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(ascii_char_variants, ascii_char)]
    /// # use sl_std::ascii;
    ///
    /// let foo: &ascii::Str = "foo".try_into().unwrap();
    /// assert!(!foo.is_empty());
    ///
    /// let bar: &ascii::Str = "".try_into().unwrap();
    /// assert!(bar.is_empty());
    /// ```
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Construct [Self] from a `&[Char]`
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(ascii_char_variants, ascii_char)]
    /// # use sl_std::ascii;
    ///
    /// let chars = &[
    ///     ascii::Char::CapitalF,
    ///     ascii::Char::SmallO,
    ///     ascii::Char::SmallO,
    /// ];
    /// let foo = ascii::Str::from_ascii_chars(chars);
    /// assert_eq!(foo.as_str(), "Foo");
    /// ```
    #[inline]
    #[must_use]
    pub const fn from_ascii_chars(chars: &[Char]) -> &Self {
        // SAFETY: Str is guaranteed to have the same layout as [Char]
        unsafe { &*(chars as *const [Char] as *const Str) }
    }

    #[inline]
    #[must_use]
    pub fn from_ascii_chars_mut(chars: &mut [Char]) -> &mut Self {
        // SAFETY: Str is guaranteed to have the same layout as [Char]
        unsafe { &mut *(chars as *mut [Char] as *mut Str) }
    }

    #[inline]
    #[must_use]
    pub fn to_lowercase(&self) -> String {
        let chars = self.chars.iter().map(Char::to_lowercase).collect();
        String::from_chars(chars)
    }

    /// Construct [Self] from a `&[u8]`
    ///
    /// Returns `None` if the bytes are not a valid ascii-encoded
    /// character sequence.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(ascii_char_variants, ascii_char)]
    /// # use sl_std::ascii;
    ///
    /// let valid = b"foobar";
    /// let invalid = b"foo\xFFbar"; // 0xFF is not ascii
    ///
    /// assert!(ascii::Str::from_bytes(valid).is_some());
    /// assert!(ascii::Str::from_bytes(invalid).is_none());
    /// ```
    #[inline]
    #[must_use]
    pub const fn from_bytes(bytes: &[u8]) -> Option<&Self> {
        // Cannot use Option::map in a const context
        match bytes.as_ascii() {
            Some(ascii_slice) => Some(Self::from_ascii_chars(ascii_slice)),
            None => None,
        }
    }

    /// Convert to a utf8 `str`
    ///
    /// This conversion is zero-cost, because ascii is a subset of unicode.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(ascii_char_variants, ascii_char)]
    /// # use sl_std::ascii;
    /// let ascii_str: &ascii::Str = "example".try_into().unwrap();
    ///
    /// assert_eq!(ascii_str.as_str(), "example");
    /// ```
    #[inline]
    #[must_use]
    pub const fn as_str(&self) -> &str {
        self.chars.as_str()
    }

    /// Converts a ascii string slice to a byte slice. To convert the byte slice back
    /// into a string slice, use the [`from_bytes`](Self::from_bytes) function.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(ascii_char_variants, ascii_char)]
    /// # use sl_std::ascii;
    /// let ascii_str: &ascii::Str = "example".try_into().unwrap();
    ///
    /// assert_eq!(ascii_str.as_bytes(), b"example");
    /// ```
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

    pub fn starts_with<'a, P>(&'a self, pattern: P) -> bool
    where
        P: super::Pattern<'a>,
    {
        pattern.is_prefix_of(self)
    }

    /// Returns a string slice with leading and trailing whitespace removed.
    ///
    /// 'Whitespace' is defined according to the terms of the [WhatWG spec](https://infra.spec.whatwg.org/#ascii-whitespace).
    /// See [Char::is_whitespace] for more information.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(ascii_char_variants, ascii_char)]
    /// # use sl_std::ascii;
    /// let s: &ascii::Str = "\nHello\tworld\t\n".try_into().unwrap();
    ///
    /// assert_eq!("Hello\tworld\t", s.trim().as_str());
    /// ```
    #[inline]
    #[must_use = "this returns the trimmed string as a slice, without modifying the original"]
    pub fn trim(&self) -> &Self {
        self.trim_matches(Char::is_whitespace)
    }

    /// Returns a string slice with leading whitespace removed.
    ///
    /// 'Whitespace' is defined according to the terms of the [WhatWG spec](https://infra.spec.whatwg.org/#ascii-whitespace).
    /// See [Char::is_whitespace] for more information.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # #![feature(ascii_char_variants, ascii_char)]
    /// # use sl_std::ascii;
    /// let s: &ascii::Str = "\nHello\tworld\t\n".try_into().unwrap();
    ///
    /// assert_eq!("Hello\tworld\t\n", s.trim_start().as_str());
    /// ```
    #[inline]
    #[must_use = "this returns the trimmed string as a slice, without modifying the original"]
    pub fn trim_start(&self) -> &Self {
        self.trim_matches_start(Char::is_whitespace)
    }

    /// Returns a string slice with trailing whitespace removed.
    ///
    /// 'Whitespace' is defined according to the terms of the [WhatWG spec](https://infra.spec.whatwg.org/#ascii-whitespace).
    /// See [Char::is_whitespace] for more information.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # #![feature(ascii_char_variants, ascii_char)]
    /// # use sl_std::ascii;
    /// let s: &ascii::Str = "\nHello\tworld\t\n".try_into().unwrap();
    ///
    /// assert_eq!("\nHello\tworld\t", s.trim_end().as_str());
    /// ```
    #[inline]
    #[must_use = "this returns the trimmed string as a slice, without modifying the original"]
    pub fn trim_end(&self) -> &Self {
        self.trim_matches_end(Char::is_whitespace)
    }

    #[must_use = "this returns the trimmed string as a slice, without modifying the original"]
    pub fn trim_matches<'a, P>(&'a self, pattern: P) -> &Self
    where
        P: super::Pattern<'a>,
        P::Searcher: super::DoubleEndedSearcher<'a>,
    {
        let mut start = 0;
        let mut end = 0;

        let mut searcher = pattern.into_searcher(self);
        if let Some((reject_start, reject_end)) = searcher.next_reject() {
            start = reject_start;
            end = reject_end
        }

        if let Some((_, reject_end)) = searcher.next_reject_back() {
            end = reject_end;
        }

        &self[start..end]
    }

    #[inline]
    #[must_use = "this returns the trimmed string as a slice, without modifying the original"]
    pub fn trim_matches_start<'a, P>(&'a self, pattern: P) -> &Self
    where
        P: super::Pattern<'a>,
    {
        let mut searcher = pattern.into_searcher(self);
        if let Some((reject_start, _)) = searcher.next_reject() {
            &self[reject_start..]
        } else {
            self
        }
    }

    #[must_use = "this returns the trimmed string as a slice, without modifying the original"]
    pub fn trim_matches_end<'a, P>(&'a self, pattern: P) -> &Self
    where
        P: super::Pattern<'a>,
        P::Searcher: super::DoubleEndedSearcher<'a>,
    {
        let mut searcher = pattern.into_searcher(self);
        if let Some((_, reject_end)) = searcher.next_reject_back() {
            &self[..reject_end]
        } else {
            self
        }
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
    /// ```
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

    #[inline]
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
    pub fn rfind<'a, P>(&'a self, pattern: P) -> Option<usize>
    where
        P: Pattern<'a>,
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

    /// Splits the string on the last occurrence of the specified delimiter and
    /// returns prefix before delimiter and suffix after delimiter.
    #[inline]
    #[must_use]
    pub fn rsplit_once<'a, P>(&'a self, delimiter: P) -> Option<(&Self, &Self)>
    where
        P: Pattern<'a>,
        P::Searcher: ReverseSearcher<'a>,
    {
        let (start, end) = delimiter.into_searcher(self).next_match_back()?;
        Some((&self[..start], &self[end..]))
    }

    #[inline]
    #[must_use]
    pub fn split_at(&self, index: usize) -> (&Self, &Self) {
        (&self[..index], &self[index..])
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

impl<'a> Default for &'a Str {
    /// Create an empty [Str]
    fn default() -> Self {
        Str::EMPTY
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

/// An iterator over match indices in a string slice.
///
///
/// This struct is created by the [`match_indices`](Str::match_indices) method on [`ascii::Str`](Str).
/// See its documentation for more.
#[derive(Clone, Copy, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
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

/// An iterator over segments of a string slice.
///
///
/// This struct is created by the [`split`](Str::split) method on [`ascii::Str`](Str).
/// See its documentation for more.
#[derive(Clone, Copy, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
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
