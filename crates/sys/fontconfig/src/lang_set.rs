use std::ffi;

use crate::bindings::{self, FcLangSet};

pub struct LangSet {
    ptr: *mut FcLangSet,
}

impl LangSet {
    #[must_use]
    pub(crate) const fn from_ptr(ptr: *mut FcLangSet) -> Self {
        Self { ptr }
    }

    /// Constructs an empty language set
    #[must_use]
    pub fn new() -> Self {
        let ptr = unsafe { bindings::FcLangSetCreate() };
        Self::from_ptr(ptr)
    }

    pub fn contains_language(&self, lang: &str) -> bool {
        let c_str = ffi::CString::new(lang).expect("null byte inside language name");

        let result = unsafe { bindings::FcLangSetHasLang(self.ptr, c_str.as_ptr()) };

        // TODO: There is some context being lost here. It would be nice if we could
        //       accurately represent the other FcLangResult values
        result == bindings::FcLangResult::FcLangEqual
    }
}

impl Clone for LangSet {
    fn clone(&self) -> Self {
        let new_ptr = unsafe { bindings::FcLangSetCopy(self.ptr) };
        Self::from_ptr(new_ptr)
    }
}

impl Drop for LangSet {
    fn drop(&mut self) {
        unsafe { bindings::FcLangSetDestroy(self.ptr) }
    }
}
