/// Trait for tracing active gc dom_objects
///
/// Containers should simply forward trace calls to each
/// of their referenced objects.
///
/// # Safety
///
/// [Trace] is unsafe, since failing to trace all references
/// can lead to them being garbage-collected while still in use.
pub unsafe trait Trace {
    fn trace(&self);
    fn root(&self);
    fn unroot(&self);
}

/// Used to impl an empty trace implementation for types that can never
/// contain `Gc<T>` types.
macro_rules! empty_trace {
    ($($type: ty,)*) => {
        $(
            unsafe impl Trace for $type {
                fn trace(&self) {}
                fn root(&self) {}
                fn unroot(&self) {}
            }
        )*
    };
}

empty_trace!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, bool, char, str, String,
);

unsafe impl<T: Trace, const N: usize> Trace for [T; N] {
    fn trace(&self) {
        for i in self {
            i.trace();
        }
    }

    fn root(&self) {
        for i in self {
            i.root();
        }
    }

    fn unroot(&self) {
        for i in self {
            i.unroot();
        }
    }
}

macro_rules! tuple_impls {
    ( $( $name:ident )+ ) => {
        #[allow(non_snake_case)]
        unsafe impl<$($name: Trace),+> Trace for ($($name,)+)
        {
            fn trace(&self) {
                let ($($name,)+) = self;
                $($name.trace();)+
            }

            fn root(&self) {
                let ($($name,)+) = self;
                $($name.root();)+
            }

            fn unroot(&self) {
                let ($($name,)+) = self;
                $($name.unroot();)+
            }
        }
    };
}

tuple_impls! { A }
tuple_impls! { A B }
tuple_impls! { A B C }
tuple_impls! { A B C D }
tuple_impls! { A B C D E }
tuple_impls! { A B C D E F }
tuple_impls! { A B C D E F G }
tuple_impls! { A B C D E F G H }
tuple_impls! { A B C D E F G H I }
tuple_impls! { A B C D E F G H I J }
tuple_impls! { A B C D E F G H I J K }
tuple_impls! { A B C D E F G H I J K L }

unsafe impl<T: Trace> Trace for Box<T> {
    fn trace(&self) {
        T::trace(self)
    }

    fn root(&self) {
        T::root(self)
    }

    fn unroot(&self) {
        T::unroot(self)
    }
}

unsafe impl<T: Trace> Trace for Option<T> {
    fn trace(&self) {
        if let Some(value) = self.as_ref() {
            value.trace();
        }
    }

    fn root(&self) {
        if let Some(value) = self.as_ref() {
            value.root();
        }
    }

    fn unroot(&self) {
        if let Some(value) = self.as_ref() {
            value.unroot();
        }
    }
}

unsafe impl<T: Trace> Trace for Vec<T> {
    fn trace(&self) {
        for i in self.iter() {
            i.trace();
        }
    }

    fn root(&self) {
        for i in self.iter() {
            i.root();
        }
    }

    fn unroot(&self) {
        for i in self.iter() {
            i.unroot();
        }
    }
}
