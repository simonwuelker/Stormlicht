/// Trait for tracing active gc dom_objects
///
/// Containers should simply forward trace calls to each
/// of their referenced objects.
///
/// [Trace] is unsafe, since failing to trace all references
/// can lead to them being garbage-collected while still in use.
pub unsafe trait Trace {
    fn trace(&self);
}

/// Used to impl an empty trace implementation for types that can never
/// contain `Gc<T>` types.
macro_rules! empty_trace {
    ($($type: ty,)*) => {
        $(
            unsafe impl Trace for $type {
                fn trace(&self) {}
            }
        )*
    };
}

empty_trace!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, bool, char, str, String,
);
