use core::ffi;
use std::ptr;

use crate::{bindings, Object};

#[repr(transparent)]
pub struct Pattern {
    ptr: *mut bindings::FcPattern,
}

#[derive(Clone, Copy, Debug)]
pub enum LookupError {
    FcResultNoMatch,
    FcResultTypeMismatch,
    FcResultNoId,
    FcResultOutOfMemory,
}

impl Pattern {
    pub fn debug_print(&self) {
        // SAFETY: ptr is guaranteed to be a valid FcPattern pointer
        unsafe { bindings::FcPatternPrint(self.ptr) }
    }

    pub fn to_string(&self) -> &ffi::CStr {
        // SAFETY: FcNameUnparse is safe, assuming our ptr is valid
        let ptr = unsafe { bindings::FcNameUnparse(self.ptr) };

        // SAFETY: fontconfig is guaranteed (assumed) to always return valid strings
        unsafe { ffi::CStr::from_ptr(ptr) }
    }

    pub fn object_count(&self) -> usize {
        let num = unsafe { bindings::FcPatternObjectCount(self.ptr) };
        debug_assert!(!num.is_negative());

        num as usize
    }

    pub(crate) fn as_ptr(&self) -> *mut bindings::FcPattern {
        self.ptr
    }

    pub fn get_string(&self, key: Object) -> Result<&ffi::CStr, LookupError> {
        let mut result_ptr = std::ptr::null();
        let return_code = unsafe {
            bindings::FcPatternGetString(self.ptr, key.as_ptr(), 0, ptr::addr_of_mut!(result_ptr))
        };
        match return_code {
            bindings::FcResult::FcResultMatch => {
                let result = unsafe { ffi::CStr::from_ptr(result_ptr) };
                Ok(result)
            },
            bindings::FcResult::FcResultNoMatch => Err(LookupError::FcResultNoMatch),
            bindings::FcResult::FcResultTypeMismatch => Err(LookupError::FcResultTypeMismatch),
            bindings::FcResult::FcResultNoId => Err(LookupError::FcResultNoId),
            bindings::FcResult::FcResultOutOfMemory => Err(LookupError::FcResultOutOfMemory),
        }
    }
}

impl Default for Pattern {
    fn default() -> Self {
        // SAFETY: FcPatternCreate is not unsafe
        let ptr = unsafe { bindings::FcPatternCreate() };

        Self { ptr }
    }
}

impl Drop for Pattern {
    fn drop(&mut self) {
        // SAFETY: FcPatternDestroy is not unsafe
        unsafe { bindings::FcPatternDestroy(self.ptr) }
    }
}
