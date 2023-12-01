use std::{ffi, mem::MaybeUninit, ptr};

use crate::{
    bindings::{self, fcbool},
    Object, Value,
};

#[repr(transparent)]
pub struct Pattern {
    ptr: *mut bindings::FcPattern,
}

impl Pattern {
    pub(crate) fn from_ptr(ptr: *mut bindings::FcPattern) -> Self {
        Self { ptr }
    }

    pub fn debug_print(&self) {
        // SAFETY: ptr is guaranteed to be a valid FcPattern pointer
        unsafe { bindings::FcPatternPrint(self.ptr) }
    }

    pub fn to_str(&self) -> &str {
        // SAFETY: FcNameUnparse is safe, assuming our ptr is valid
        let ptr = unsafe { bindings::FcNameUnparse(self.ptr) };

        // SAFETY: fontconfig is guaranteed (assumed) to always return valid strings
        unsafe { bindings::to_str(ptr) }
    }

    pub fn object_count(&self) -> usize {
        let num = unsafe { bindings::FcPatternObjectCount(self.ptr) };
        debug_assert!(!num.is_negative());

        num as usize
    }

    pub(crate) fn as_ptr(&self) -> *mut bindings::FcPattern {
        self.ptr
    }

    pub fn get_string(&self, key: Object) -> Result<&str, bindings::LookupError> {
        let mut result_ptr = std::ptr::null();
        let return_code = unsafe {
            bindings::FcPatternGetString(self.ptr, key.as_ptr(), 0, ptr::addr_of_mut!(result_ptr))
        };

        return_code.to_rust_result(|| unsafe { bindings::to_str(result_ptr) })
    }

    pub fn get(&self, key: Object) -> Result<Value, bindings::LookupError> {
        let mut result = MaybeUninit::uninit();
        let return_code =
            unsafe { bindings::FcPatternGet(self.ptr, key.as_ptr(), 0, result.as_mut_ptr()) };

        return_code.to_rust_result(|| unsafe { result.assume_init() }.into())
    }

    pub fn get_int(&self, key: Object) -> Result<i32, bindings::LookupError> {
        let mut result = 0;
        let return_code =
            unsafe { bindings::FcPatternGetInteger(self.ptr, key.as_ptr(), 0, &mut result) };

        return_code.to_rust_result(|| result)
    }

    pub fn add_string(&self, key: Object, value: &str) {
        let c_string = ffi::CString::new(value).expect("null byte inside value");
        // NOTE: fontconfig keeps no references to the passed string, so it is safe to drop it at the end
        //       of this function
        let success =
            unsafe { bindings::FcPatternAddString(self.as_ptr(), key.as_ptr(), c_string.as_ptr()) };

        if !fcbool(success) {
            panic!("failed to insert {key:?} with value {value:?} into pattern");
        }
    }

    pub fn add_int(&self, key: Object, value: i32) {
        let success = unsafe { bindings::FcPatternAddInteger(self.as_ptr(), key.as_ptr(), value) };

        if !fcbool(success) {
            panic!("failed to insert {key:?} with value {value:?} into pattern");
        }
    }

    /// Calls [fcdefaultsubstitute](https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcdefaultsubstitute.html) on the pattern
    pub fn perform_default_substitutions(&self) {
        unsafe { bindings::FcDefaultSubstitute(self.as_ptr()) }
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
