use crate::bindings;

pub struct Pattern {
    ptr: *mut bindings::FcPattern,
}

impl Pattern {
    pub fn debug_print(&self) {
        // SAFETY: ptr is guaranteed to be a valid FcPattern pointer
        unsafe { bindings::FcPatternPrint(self.ptr) }
    }

    pub fn to_string(&self) -> super::String {
        // SAFETY: FcNameUnparse is safe, assuming our ptr is valid
        let ptr = unsafe { bindings::FcNameUnparse(self.ptr) };

        // SAFETY: fontconfig is guaranteed (assumed) to always return valid strings
        unsafe { super::String::from_ptr(ptr) }
    }
}

impl Default for Pattern {
    fn default() -> Self {
        // SAFETY: FcPatternCreate is not unsafe
        let ptr = unsafe { bindings::FcPatternCreate() };

        Self { ptr }
    }
}
