use std::{io, mem};

macro_rules! impl_read_for_int {
    ($int: ident, $be_fn_name: ident, $le_fn_name: ident) => {
        fn $be_fn_name(&mut self) -> io::Result<$int> {
            let mut buf = [0; mem::size_of::<$int>()];
            self.read_exact(&mut buf)?;
            Ok($int::from_be_bytes(buf))
        }

        fn $le_fn_name(&mut self) -> io::Result<$int> {
            let mut buf = [0; mem::size_of::<$int>()];
            self.read_exact(&mut buf)?;
            Ok($int::from_le_bytes(buf))
        }
    };
}
pub trait ReadExt: io::Read {
    // Technically not necessary, but lets add endianness functions
    // for u8/i8 anyways for consistency
    impl_read_for_int!(u8, read_be_u8, read_le_u8);
    impl_read_for_int!(i8, read_be_i8, read_le_i8);

    impl_read_for_int!(u16, read_be_u16, read_le_u16);
    impl_read_for_int!(i16, read_be_i16, read_le_i16);

    impl_read_for_int!(u32, read_be_u32, read_le_u32);
    impl_read_for_int!(i32, read_be_i32, read_le_i32);

    impl_read_for_int!(u64, read_be_u64, read_le_u64);
    impl_read_for_int!(i64, read_be_i64, read_le_i64);

    impl_read_for_int!(u128, read_be_u128, read_le_u128);
    impl_read_for_int!(i128, read_be_i128, read_le_i128);

    fn read_value<T: Readable>(&mut self) -> Result<T, T::Error> {
        T::read(self)
    }
}

impl<T: io::Read> ReadExt for T {}

/// Marks types that can be created from a `Read` instance
pub trait Readable: Sized {
    type Error;

    /// Deserialize the type from a read instance
    fn read<R: io::Read>(reader: R) -> Result<Self, Self::Error>;
}
