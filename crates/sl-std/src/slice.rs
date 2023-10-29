use crate::range::Range;

pub trait SubsliceOffset {
    /// Returns the byte offset of an inner slice relative to an enclosing outer slice.
    ///
    /// Examples
    ///
    /// ```
    /// # use sl_std::slice::SubsliceOffset;
    ///
    /// let string = "a\nb\nc";
    /// let lines: Vec<&str> = string.lines().collect();
    /// assert!(string.subslice_offset(lines[0]) == Some(0)); // &"a"
    /// assert!(string.subslice_offset(lines[1]) == Some(2)); // &"b"
    /// assert!(string.subslice_offset(lines[2]) == Some(4)); // &"c"
    /// assert!(string.subslice_offset("other!") == None);
    /// ```
    fn subslice_offset(&self, inner: &Self) -> Option<usize>;

    fn subslice_range(&self, inner: &Self) -> Option<Range<usize>>;
}

impl SubsliceOffset for str {
    fn subslice_offset(&self, inner: &str) -> Option<usize> {
        let outer = self.as_ptr() as usize;
        let inner = inner.as_ptr() as usize;
        if (outer..=outer + self.len()).contains(&inner) {
            Some(inner.wrapping_sub(outer))
        } else {
            None
        }
    }

    fn subslice_range(&self, inner: &Self) -> Option<Range<usize>> {
        let start = self.subslice_offset(inner)?;
        let outer = self.as_ptr() as usize;
        let end = start + inner.len();

        if outer + self.len() < end {
            return None;
        }

        Some(Range::new(start, end))
    }
}
