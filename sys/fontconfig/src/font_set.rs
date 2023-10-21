use std::{marker::PhantomData, mem};

use crate::{
    bindings::{self, SetIterator},
    Pattern,
};

pub struct FontSet {
    ptr: *mut bindings::FcFontSet,
}

impl FontSet {
    /// Create a new fontset from a raw pointer
    ///
    /// # Safety
    /// Behaviour is undefined if the pointer does not point to a valid [FcFontSet](bindings::FcFontSet)
    pub unsafe fn from_ptr(ptr: *mut bindings::FcFontSet) -> Self {
        debug_assert!(!ptr.is_null());
        Self { ptr }
    }

    pub fn debug_print(&self) {
        // SAFETY: FcFontSetPrint is not unsafe
        unsafe { bindings::FcFontSetPrint(self.ptr) }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Pattern> {
        let set = self.as_ref();
        let iter = SetIterator {
            current: set.value,
            remaining: set.num_values as usize,
            phantom_data: PhantomData,
        };

        iter.into_iter().map(|pattern_ptr| {
            // SAFETY: Pattern is guaranteed to have the same layout and alignment as
            //         *mut FcPattern because of #[repr(transparent)]
            unsafe { mem::transmute(pattern_ptr) }
        })
    }

    /// Returns the number of elements in the font set
    pub fn len(&self) -> usize {
        self.as_ref().num_values as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl AsRef<bindings::FcFontSet> for FontSet {
    fn as_ref(&self) -> &bindings::FcFontSet {
        // SAFETY: self.ptr is guaranteed to be valid
        unsafe { &*self.ptr }
    }
}
