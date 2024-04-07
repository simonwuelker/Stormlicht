//! Safe, infallible casts

use std::{mem, ptr};

/// Marker traits for types that are simply data that cannot be in an invalid state and doesn't have padding
pub unsafe trait Plain {
    fn cast<P: Plain>(self) -> P
    where
        Self: Sized,
    {
        assert!(mem::size_of::<P>() == mem::size_of::<Self>());

        let result_ptr = ptr::NonNull::from(&self).cast();

        assert!(result_ptr.is_aligned());

        // SAFETY: Pointer is known to be valid regarding size/alignment
        //         P has no "invalid state" so initializing it is always safe
        unsafe { result_ptr.read() }
    }
}

macro_rules! impl_plain {
    ($($name: ty,)*) => {
        $(
            unsafe impl Plain for $name {}
        )*
    };
}

impl_plain!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128,);

unsafe impl<T: Plain, const N: usize> Plain for [T; N] {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cast_between_integers() {
        let x: u32 = 0xdeadbeef;
        let y: [u16; 2] = x.cast();
        let z: [u8; 4] = y.cast();

        assert_eq!(z, x.to_ne_bytes());
    }

    #[test]
    #[should_panic]
    fn cast_to_different_size() {
        let x: u32 = 0xdeadbeef;
        let _: u64 = x.cast();
    }
}
