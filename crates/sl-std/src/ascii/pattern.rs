//! The ascii string pattern API
//!
//! Largely the same as the one in [std](https://doc.rust-lang.org/std/str/pattern/index.html), except it's for
//! [&'a ascii::Str(super::Str) instead of regular [&'a str](str)

/// Result of calling [`Searcher::next()`] or [`ReverseSearcher::next_back()`].
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum SearchStep {
    /// Expresses that a match of the pattern has been found at
    /// `haystack[a..b]`.
    Match(usize, usize),
    /// Expresses that `haystack[a..b]` has been rejected as a possible match
    /// of the pattern.
    ///
    /// Note that there might be more than one `Reject` between two `Match`es,
    /// there is no requirement for them to be combined into one.
    Reject(usize, usize),
    /// Expresses that every byte of the haystack has been visited, ending
    /// the iteration.
    Done,
}

/// A string pattern.
///
/// A `Pattern<'a>` expresses that the implementing type
/// can be used as a string pattern for searching in a [`&'a ascii::Str`](super::Str).
///
/// For example, both `'a'` and `"aa"` are patterns that
/// would match at index `1` in the string `"baaaab"`.
///
/// The trait itself acts as a builder for an associated
/// [`Searcher`] type, which does the actual work of finding
/// occurrences of the pattern in a string.
///
/// Depending on the type of the pattern, the behaviour of methods like
/// [`ascii::Str::find`](super::Str::find) and [`str::split`](super::Str::split) can change.
/// The table below describes some of those behaviours.
///
/// | Pattern type             | Match condition                           |
/// |--------------------------|-------------------------------------------|
/// | `ascii::Char`            | is contained in string                    |
pub trait Pattern<'a>: Sized {
    /// Associated searcher for this pattern
    type Searcher: Searcher<'a>;

    /// Constructs the associated searcher from
    /// `self` and the `haystack` to search in.
    fn into_searcher(self, haystack: &'a super::Str) -> Self::Searcher;

    /// Checks whether the pattern matches anywhere in the haystack
    #[inline]
    #[allow(clippy::wrong_self_convention)]
    fn is_contained_in(self, haystack: &'a super::Str) -> bool {
        self.into_searcher(haystack).next_match().is_some()
    }

    /// Checks whether the pattern matches at the front of the haystack
    #[inline]
    #[allow(clippy::wrong_self_convention)]
    fn is_prefix_of(self, haystack: &'a super::Str) -> bool {
        matches!(self.into_searcher(haystack).next(), SearchStep::Match(0, _))
    }

    /// Checks whether the pattern matches at the back of the haystack
    #[inline]
    #[allow(clippy::wrong_self_convention)]
    fn is_suffix_of(self, haystack: &'a super::Str) -> bool
    where
        Self::Searcher: ReverseSearcher<'a>,
    {
        matches!(self.into_searcher(haystack).next_back(), SearchStep::Match(_, j) if haystack.len() == j)
    }

    /// Removes the pattern from the front of haystack, if it matches.
    #[inline]
    fn strip_prefix_of(self, haystack: &'a super::Str) -> Option<&'a super::Str> {
        if let SearchStep::Match(start, len) = self.into_searcher(haystack).next() {
            debug_assert_eq!(
                start, 0,
                "The first search step from Searcher \
                 must include the first character"
            );
            Some(&haystack[len..])
        } else {
            None
        }
    }

    /// Removes the pattern from the back of haystack, if it matches.
    #[inline]
    fn strip_suffix_of(self, haystack: &'a super::Str) -> Option<&'a super::Str>
    where
        Self::Searcher: ReverseSearcher<'a>,
    {
        if let SearchStep::Match(start, end) = self.into_searcher(haystack).next_back() {
            debug_assert_eq!(
                end,
                haystack.len(),
                "The first search step from ReverseSearcher \
                 must include the last character"
            );
            Some(&haystack[..start])
        } else {
            None
        }
    }
}

/// A searcher for a string pattern.
///
/// This trait provides methods for searching for non-overlapping
/// matches of a pattern starting from the front (left) of a string.
///
/// It will be implemented by associated `Searcher`
/// types of the [`Pattern`] trait.
pub trait Searcher<'a> {
    /// Getter for the underlying string to be searched in
    ///
    /// Will always return the same [`&ascii::Str`][super::Str].
    fn haystack(&self) -> &'a super::Str;

    /// Performs the next search step starting from the front.
    ///
    /// - Returns [`Match(a, b)`][SearchStep::Match] if `haystack[a..b]` matches
    ///   the pattern.
    /// - Returns [`Reject(a, b)`][SearchStep::Reject] if `haystack[a..b]` can
    ///   not match the pattern, even partially.
    /// - Returns [`Done`][SearchStep::Done] if every byte of the haystack has
    ///   been visited.
    ///
    /// The stream of [`Match`][SearchStep::Match] and
    /// [`Reject`][SearchStep::Reject] values up to a [`Done`][SearchStep::Done]
    /// will contain index ranges that are adjacent, non-overlapping,
    /// covering the whole haystack, and laying on utf8 boundaries.
    ///
    /// A [`Match`][SearchStep::Match] result needs to contain the whole matched
    /// pattern, however [`Reject`][SearchStep::Reject] results may be split up
    /// into arbitrary many adjacent fragments. Both ranges may have zero length.
    ///
    /// As an example, the pattern `"aaa"` and the haystack `"cbaaaaab"`
    /// might produce the stream
    /// `[Reject(0, 1), Reject(1, 2), Match(2, 5), Reject(5, 8)]`
    fn next(&mut self) -> SearchStep;

    /// Finds the next [`Match`][SearchStep::Match] result. See [`next()`][Searcher::next].
    ///
    /// Unlike [`next()`][Searcher::next], there is no guarantee that the returned ranges
    /// of this and [`next_reject`][Searcher::next_reject] will overlap. This will return
    /// `(start_match, end_match)`, where start_match is the index of where
    /// the match begins, and end_match is the index after the end of the match.
    #[inline]
    fn next_match(&mut self) -> Option<(usize, usize)> {
        loop {
            match self.next() {
                SearchStep::Match(a, b) => return Some((a, b)),
                SearchStep::Done => return None,
                _ => continue,
            }
        }
    }

    /// Finds the next [`Reject`][SearchStep::Reject] result. See [`next()`][Searcher::next]
    /// and [`next_match()`][Searcher::next_match].
    ///
    /// Unlike [`next()`][Searcher::next], there is no guarantee that the returned ranges
    /// of this and [`next_match`][Searcher::next_match] will overlap.
    #[inline]
    fn next_reject(&mut self) -> Option<(usize, usize)> {
        loop {
            match self.next() {
                SearchStep::Reject(a, b) => return Some((a, b)),
                SearchStep::Done => return None,
                _ => continue,
            }
        }
    }
}

/// A reverse searcher for a string pattern.
///
/// This trait provides methods for searching for non-overlapping
/// matches of a pattern starting from the back (right) of a string.
///
/// It will be implemented by associated [`Searcher`]
/// types of the [`Pattern`] trait if the pattern supports searching
/// for it from the back.
///
/// The index ranges returned by this trait are not required
/// to exactly match those of the forward search in reverse.
pub trait ReverseSearcher<'a>: Searcher<'a> {
    /// Performs the next search step starting from the back.
    ///
    /// - Returns [`Match(a, b)`][SearchStep::Match] if `haystack[a..b]`
    ///   matches the pattern.
    /// - Returns [`Reject(a, b)`][SearchStep::Reject] if `haystack[a..b]`
    ///   can not match the pattern, even partially.
    /// - Returns [`Done`][SearchStep::Done] if every byte of the haystack
    ///   has been visited
    ///
    /// The stream of [`Match`][SearchStep::Match] and
    /// [`Reject`][SearchStep::Reject] values up to a [`Done`][SearchStep::Done]
    /// will contain index ranges that are adjacent, non-overlapping,
    /// covering the whole haystack, and laying on utf8 boundaries.
    ///
    /// A [`Match`][SearchStep::Match] result needs to contain the whole matched
    /// pattern, however [`Reject`][SearchStep::Reject] results may be split up
    /// into arbitrary many adjacent fragments. Both ranges may have zero length.
    ///
    /// As an example, the pattern `"aaa"` and the haystack `"cbaaaaab"`
    /// might produce the stream
    /// `[Reject(7, 8), Match(4, 7), Reject(1, 4), Reject(0, 1)]`.
    fn next_back(&mut self) -> SearchStep;

    /// Finds the next [`Match`][SearchStep::Match] result.
    /// See [`next_back()`][ReverseSearcher::next_back].
    #[inline]
    fn next_match_back(&mut self) -> Option<(usize, usize)> {
        loop {
            match self.next_back() {
                SearchStep::Match(a, b) => return Some((a, b)),
                SearchStep::Done => return None,
                _ => continue,
            }
        }
    }

    /// Finds the next [`Reject`][SearchStep::Reject] result.
    /// See [`next_back()`][ReverseSearcher::next_back].
    #[inline]
    fn next_reject_back(&mut self) -> Option<(usize, usize)> {
        loop {
            match self.next_back() {
                SearchStep::Reject(a, b) => return Some((a, b)),
                SearchStep::Done => return None,
                _ => continue,
            }
        }
    }
}

/// A marker trait to express that a [`ReverseSearcher`]
/// can be used for a [`DoubleEndedIterator`] implementation.
///
/// For this, the impl of [`Searcher`] and [`ReverseSearcher`] need
/// to follow these conditions:
///
/// - All results of `next()` need to be identical
///   to the results of `next_back()` in reverse order.
/// - `next()` and `next_back()` need to behave as
///   the two ends of a range of values, that is they
///   can not "walk past each other".
///
/// # Examples
///
/// `ascii::Char::Searcher` is a `DoubleEndedSearcher` because searching for a
/// [`char`] only requires looking at one at a time, which behaves the same
/// from both ends.
pub trait DoubleEndedSearcher<'a>: ReverseSearcher<'a> {}

pub trait SingleCharEq {
    fn matches_char(&self, c: &super::Char) -> bool;
}

impl SingleCharEq for super::Char {
    fn matches_char(&self, c: &super::Char) -> bool {
        self == c
    }
}

impl<F> SingleCharEq for F
where
    F: Fn(&super::Char) -> bool,
{
    fn matches_char(&self, c: &super::Char) -> bool {
        self(c)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AsciiCharSearcher<'a, T>
where
    T: SingleCharEq,
{
    haystack: &'a super::Str,
    pos_front: usize,
    pos_back: usize,
    needle: T,
}

impl<'a, F> Pattern<'a> for F
where
    F: Fn(&super::Char) -> bool,
{
    type Searcher = AsciiCharSearcher<'a, F>;

    fn into_searcher(self, haystack: &'a super::Str) -> Self::Searcher {
        AsciiCharSearcher {
            haystack,
            pos_front: 0,
            pos_back: haystack.len(),
            needle: self,
        }
    }
}

impl<'a> Pattern<'a> for super::Char {
    type Searcher = AsciiCharSearcher<'a, Self>;

    fn into_searcher(self, haystack: &'a super::Str) -> Self::Searcher {
        AsciiCharSearcher {
            haystack,
            pos_front: 0,
            pos_back: haystack.len(),
            needle: self,
        }
    }

    fn is_contained_in(self, haystack: &'a super::Str) -> bool {
        haystack.chars().contains(&self)
    }

    fn is_prefix_of(self, haystack: &'a super::Str) -> bool {
        haystack.chars().first().is_some_and(|&c| c == self)
    }

    fn is_suffix_of(self, haystack: &'a super::Str) -> bool
    where
        Self::Searcher: ReverseSearcher<'a>,
    {
        haystack.chars().last().is_some_and(|&c| c == self)
    }

    fn strip_prefix_of(self, haystack: &'a super::Str) -> Option<&'a super::Str> {
        if self.is_prefix_of(haystack) {
            Some(&haystack[1..])
        } else {
            None
        }
    }

    fn strip_suffix_of(self, haystack: &'a super::Str) -> Option<&'a super::Str>
    where
        Self::Searcher: ReverseSearcher<'a>,
    {
        if self.is_suffix_of(haystack) {
            Some(&haystack[..haystack.len() - 1])
        } else {
            None
        }
    }
}

impl<'a, T> Searcher<'a> for AsciiCharSearcher<'a, T>
where
    T: SingleCharEq,
{
    fn haystack(&self) -> &'a super::Str {
        self.haystack
    }

    fn next(&mut self) -> SearchStep {
        let remaining = &self.haystack[self.pos_front..self.pos_back];
        if remaining.is_empty() {
            return SearchStep::Done;
        }

        let index = remaining
            .chars()
            .iter()
            .position(|c| self.needle.matches_char(c));

        match index {
            None => {
                // No more matches in the remainder of the haystack
                let reject_range = SearchStep::Reject(self.pos_front, self.pos_back);
                self.pos_front = self.pos_back;
                reject_range
            },
            Some(0) => {
                let match_range = SearchStep::Match(self.pos_front, self.pos_front + 1);
                self.pos_front += 1;
                match_range
            },
            Some(nonzero_index) => {
                let reject_range =
                    SearchStep::Reject(self.pos_front, self.pos_front + nonzero_index);
                self.pos_front += nonzero_index;
                reject_range
            },
        }
    }
}

impl<'a, T> ReverseSearcher<'a> for AsciiCharSearcher<'a, T>
where
    T: SingleCharEq,
{
    fn next_back(&mut self) -> SearchStep {
        let remaining = &self.haystack[self.pos_front..self.pos_back];
        if remaining.is_empty() {
            return SearchStep::Done;
        }

        let index = remaining
            .chars()
            .iter()
            .rev()
            .position(|c| self.needle.matches_char(c));

        match index {
            None => {
                // No more matches in the remainder of the haystack
                let reject_range = SearchStep::Reject(self.pos_front, self.pos_back);
                self.pos_back = self.pos_front;
                reject_range
            },
            Some(0) => {
                let match_range = SearchStep::Match(self.pos_back - 1, self.pos_back);
                self.pos_back -= 1;
                match_range
            },
            Some(nonzero_index) => {
                let reject_range = SearchStep::Reject(self.pos_back - nonzero_index, self.pos_back);
                self.pos_back -= nonzero_index;
                reject_range
            },
        }
    }
}

impl<'a, T> DoubleEndedSearcher<'a> for AsciiCharSearcher<'a, T> where T: SingleCharEq {}

/// A [ascii::Searcher](Searcher) used by various byte-sequence types ([&str](str) and [&ascii::Str](super::Str))
///
/// Generally, searching for non-ascii data in a [&ascii::Str](super::Str) will yield no matches.
#[derive(Clone, Copy, Debug)]
pub struct BytesSearcher<'a> {
    haystack: &'a super::Str,
    pos_front: usize,
    pos_back: usize,
    needle: &'a [u8],
}

impl<'a> Pattern<'a> for &'a super::Str {
    type Searcher = BytesSearcher<'a>;

    fn into_searcher(self, haystack: &'a super::Str) -> Self::Searcher {
        BytesSearcher {
            haystack,
            pos_front: 0,
            pos_back: haystack.len(),
            needle: self.as_bytes(),
        }
    }
}

impl<'a> Pattern<'a> for &'a str {
    type Searcher = BytesSearcher<'a>;

    fn into_searcher(self, haystack: &'a super::Str) -> Self::Searcher {
        BytesSearcher {
            haystack,
            pos_front: 0,
            pos_back: haystack.len(),
            needle: self.as_bytes(),
        }
    }
}

impl<'a> Pattern<'a> for &'a [super::Char] {
    type Searcher = BytesSearcher<'a>;

    fn into_searcher(self, haystack: &'a super::Str) -> Self::Searcher {
        super::Str::from_ascii_chars(self).into_searcher(haystack)
    }
}

impl<'a> Searcher<'a> for BytesSearcher<'a> {
    fn haystack(&self) -> &'a super::Str {
        self.haystack
    }

    fn next(&mut self) -> SearchStep {
        let remaining = &self.haystack[self.pos_front..self.pos_back];
        if remaining.is_empty() {
            return SearchStep::Done;
        }

        let index = remaining
            .as_bytes()
            .windows(self.needle.len())
            .position(|window| window == self.needle);

        match index {
            None => {
                let reject_range = SearchStep::Reject(self.pos_front, self.pos_back);
                self.pos_front = self.pos_back;
                reject_range
            },
            Some(0) => {
                let match_range =
                    SearchStep::Match(self.pos_front, self.pos_front + self.needle.len());
                self.pos_front += self.needle.len();
                match_range
            },
            Some(i) => {
                let reject_range = SearchStep::Reject(self.pos_front, self.pos_front + i);
                self.pos_front += i;
                reject_range
            },
        }
    }
}

impl<'a> ReverseSearcher<'a> for BytesSearcher<'a> {
    fn next_back(&mut self) -> SearchStep {
        let remaining = &self.haystack[self.pos_front..self.pos_back];
        if remaining.is_empty() {
            return SearchStep::Done;
        }

        let index_from_back = remaining
            .as_bytes()
            .windows(self.needle.len())
            .rev()
            .position(|window| window == self.needle);

        match index_from_back {
            None => {
                let reject_range = SearchStep::Reject(self.pos_front, self.pos_back);
                self.pos_back = self.pos_front;
                reject_range
            },
            Some(0) => {
                let match_range =
                    SearchStep::Match(self.pos_back - self.needle.len(), self.pos_back);
                self.pos_back -= self.needle.len();
                match_range
            },
            Some(i) => {
                let reject_range = SearchStep::Reject(self.pos_back - i, self.pos_back);
                self.pos_back -= i;
                reject_range
            },
        }
    }
}

impl<'a> DoubleEndedSearcher<'a> for BytesSearcher<'a> {}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ascii;

    #[test]
    fn ascii_char_pattern() {
        let haystack: &ascii::Str = "foobar".try_into().unwrap();

        assert!(ascii::Char::SmallF.is_prefix_of(haystack));
        assert!(ascii::Char::SmallR.is_suffix_of(haystack));

        assert_eq!(
            ascii::Char::SmallF
                .strip_prefix_of(haystack)
                .map(ascii::Str::as_str),
            Some("oobar")
        );

        assert_eq!(
            ascii::Char::SmallR
                .strip_suffix_of(haystack)
                .map(ascii::Str::as_str),
            Some("fooba")
        );

        assert_eq!(
            ascii::Char::SmallA
                .strip_suffix_of(haystack)
                .map(ascii::Str::as_str),
            None,
        );

        let mut o_searcher = ascii::Char::SmallO.into_searcher(haystack);
        assert_eq!(o_searcher.next_match(), Some((1, 2)));
        assert_eq!(o_searcher.next_match(), Some((2, 3)));
        assert_eq!(o_searcher.next_match(), None);

        let mut o_searcher = ascii::Char::SmallO.into_searcher(haystack);
        assert_eq!(o_searcher.next_match_back(), Some((2, 3)));
        assert_eq!(o_searcher.next_match_back(), Some((1, 2)));
        assert_eq!(o_searcher.next_match_back(), None);
    }

    #[test]
    fn fn_pattern() {
        let haystack: &ascii::Str = "foobar".try_into().unwrap();

        let f = |c: &ascii::Char| *c == ascii::Char::SmallF;
        let a_or_r = |c: &ascii::Char| matches!(c, ascii::Char::SmallA | ascii::Char::SmallR);

        assert!(f.is_prefix_of(haystack));
        assert!(!a_or_r.is_prefix_of(haystack));
        assert!(a_or_r.is_suffix_of(haystack));

        assert_eq!(
            f.strip_prefix_of(haystack).map(ascii::Str::as_str),
            Some("oobar")
        );

        assert_eq!(
            a_or_r.strip_suffix_of(haystack).map(ascii::Str::as_str),
            Some("fooba")
        );

        assert_eq!(f.strip_suffix_of(haystack).map(ascii::Str::as_str), None);

        let mut o_searcher = (|c: &ascii::Char| *c == ascii::Char::SmallO).into_searcher(haystack);
        assert_eq!(o_searcher.next_match(), Some((1, 2)));
        assert_eq!(o_searcher.next_match(), Some((2, 3)));
        assert_eq!(o_searcher.next_match(), None);

        let mut o_searcher = (|c: &ascii::Char| *c == ascii::Char::SmallO).into_searcher(haystack);
        assert_eq!(o_searcher.next_match_back(), Some((2, 3)));
        assert_eq!(o_searcher.next_match_back(), Some((1, 2)));
        assert_eq!(o_searcher.next_match_back(), None);
    }

    #[test]
    fn str_pattern() {
        let haystack: &ascii::Str = "Lorem ipsum dolor".try_into().unwrap();

        assert!("Lorem".is_prefix_of(haystack));
        assert!(!"lorem".is_prefix_of(haystack));
        assert!("dolor".is_suffix_of(haystack));

        assert_eq!(
            "Lorem".strip_prefix_of(haystack).map(ascii::Str::as_str),
            Some(" ipsum dolor")
        );
        assert_eq!(
            "ipsum".strip_prefix_of(haystack).map(ascii::Str::as_str),
            None
        );
        assert_eq!(
            "dolor".strip_suffix_of(haystack).map(ascii::Str::as_str),
            Some("Lorem ipsum ")
        );
    }
}
