#[macro_export]
macro_rules! const_assert {
    ($cond:expr $(,)?) => {
        const __: () = assert!($cond);
    };
    ($cond:expr, $($arg:tt)+) => {
        const __: () = assert!($cond, $($arg)*);
    };
}
