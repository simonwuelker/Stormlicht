use std::{fmt, slice};

use crate::bindings;

/// FontConfig String type
///
/// # Safety
/// [String::ptr] is guaranteed to be valid for [String::length] bytes.
pub struct String {
    /// Pointer to the string memory
    ///
    /// # Safety
    /// Guaranteed to point to a string conforming to the
    /// expectations by fontconfig
    ptr: *mut u8,
    length: usize,
}

impl String {
    /// Construct a new String from a pointer (usually received from FFI)
    ///
    /// # Safety
    /// `ptr` **must** point to a valid, null terminated string received from fontconfig
    pub(crate) unsafe fn from_ptr(ptr: *mut bindings::FcChar8) -> Self {
        let mut length = 0;

        let mut i = ptr;

        // SAFETY: ptr is assumed to point to a null-terminated byte sequence
        while unsafe { *i } != 0 {
            length += 1;
            i = i.wrapping_add(1);
        }

        Self { ptr, length }
    }

    fn as_bytes(&self) -> &[u8] {
        // SAFETY: ptr and length are guaranteed to be valid according to the safety
        //         conditions of Self
        unsafe { slice::from_raw_parts(self.ptr, self.length) }
    }

    fn as_str(&self) -> &str {
        std::str::from_utf8(self.as_bytes()).expect("fontconfig produced non-utf8 string")
    }
}

impl fmt::Debug for String {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.as_str())
    }
}

impl fmt::Display for String {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Drop for String {
    fn drop(&mut self) {
        // SAFETY: FcStrFree is not unsafe
        unsafe { bindings::FcStrFree(self.ptr) }
    }
}
