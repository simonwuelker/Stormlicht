use std::fmt;

use crate::bindings;

pub struct Range {
    ptr: *const bindings::FcRange,
}

impl Range {
    #[must_use]
    pub fn new_from_int(begin: i32, end: i32) -> Self {
        let ptr = unsafe { bindings::FcRangeCreateInteger(begin, end) };
        Self { ptr }
    }

    #[must_use]
    pub fn new_from_double(begin: f64, end: f64) -> Self {
        let ptr = unsafe { bindings::FcRangeCreateDouble(begin, end) };
        Self { ptr }
    }

    /// Takes ownership of the passed pointer.
    #[must_use]
    pub fn from_ptr(ptr: *const bindings::FcRange) -> Self {
        Self { ptr }
    }

    #[must_use]
    pub fn get_bounds(&self) -> (f64, f64) {
        let mut begin = 0.;
        let mut end = 0.;
        unsafe { bindings::FcRangeGetDouble(self.ptr, &mut begin, &mut end) };
        (begin, end)
    }
}

impl Clone for Range {
    fn clone(&self) -> Self {
        let ptr = unsafe { bindings::FcRangeCopy(self.ptr) };
        Self { ptr }
    }
}

impl fmt::Debug for Range {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (begin, end) = self.get_bounds();
        write!(f, "[{begin}, {end}]")
    }
}

impl Drop for Range {
    fn drop(&mut self) {
        unsafe { bindings::FcRangeDestroy(self.ptr) }
    }
}
