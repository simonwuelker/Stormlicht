/// The boolean `true` on the type level
#[derive(Clone, Copy, Debug)]
pub struct True;

/// The boolean `false` on the type level
#[derive(Clone, Copy, Debug)]
pub struct False;

pub trait ToBool: Sized {
    type Bool: Sized;

    const VALUE: Self::Bool;
}

impl ToBool for [(); 0] {
    type Bool = False;

    const VALUE: Self::Bool = False;
}

impl ToBool for [(); 1] {
    type Bool = True;

    const VALUE: Self::Bool = True;
}

/// Converts a const-bool expression into a type-level boolean ([True]/[False])
#[macro_export]
macro_rules! to_type_level_bool {
    ($x:expr) => {{
        const X: bool = $x;
        <[(); X as usize] as $crate::assert::ToBool>::VALUE
    }};
}

/// Assertion that is guaranteed to run at compile time
///
///	Only one const assertion is allowed per namespace.
/// If you need more, simply combine them together into one.
///
/// ## Implementation
/// The usual approach to this is the following:
/// ```rust, ignore
/// const _: () = assert!(condition());
/// ```
///
/// This is dangerous: associated constants, unlike free constants,
/// are only evaluated when referenced.
/// Thats why the following does not cause a panic:
/// ```rust, ignore
/// struct Foo;
///
/// impl Foo {
/// 	const _: () = assert!(false);
/// }
/// ```
///
/// To circumvent this, we work with booleans on the type-system level.
/// This works because while the code does not get evaluated, type inference
/// is still run.
#[macro_export]
macro_rules! const_assert {
    ($cond:expr $(,)?) => {
        #[doc(hidden)]
        const __: $crate::assert::True = $crate::to_type_level_bool!($cond);
    };
}
