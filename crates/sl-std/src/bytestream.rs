//! Provides a [Cursor](std::io::Cursor) equivalent without [io::Error](std::io::Error)

#[derive(Clone, Copy)]
/// Provides a [Cursor](std::io::Cursor) equivalent without [io::Error](std::io::Error)
pub struct ByteStream<'a> {
    bytes: &'a [u8],
    cursor: usize,
}

macro_rules! next_int {
    ($primitive: ty, $len: expr, $be_function: ident, $le_function: ident) => {
        #[must_use]
        pub fn $be_function(&mut self) -> Option<$primitive> {
            self.next_chunk().map(<$primitive>::from_be_bytes)
        }

        #[must_use]
        pub fn $le_function(&mut self) -> Option<$primitive> {
            self.next_chunk().map(<$primitive>::from_le_bytes)
        }
    };
}

impl<'a> ByteStream<'a> {
    #[must_use]
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, cursor: 0 }
    }

    /// Returns the bytes from cursor until the end of the stream
    ///
    /// If the cursor is past the end of the stream, an empty slice is returned
    #[must_use]
    pub fn remaining(&self) -> &[u8] {
        let index = self.cursor.min(self.bytes.len() - 1);
        &self.bytes[index..]
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.remaining().is_empty()
    }

    pub fn advance(&mut self, n: usize) {
        self.cursor += n;
    }

    #[must_use]
    pub const fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn set_cursor(&mut self, cursor: usize) {
        self.cursor = cursor;
    }

    #[inline]
    #[must_use]
    pub fn next_chunk<const N: usize>(&mut self) -> Option<[u8; N]> {
        let remaining = self.remaining();

        if remaining.len() < N {
            return None;
        }

        let chunk = remaining[..N]
            .try_into()
            .expect("Slice is exactly N elements long");

        self.cursor += N;

        Some(chunk)
    }

    #[must_use]
    pub fn next_byte(&mut self) -> Option<u8> {
        let byte = self.remaining().get(0).copied();
        self.cursor += 1;
        byte
    }

    next_int!(u16, 2, next_be_u16, next_le_u16);
    next_int!(i16, 2, next_be_i16, next_le_i16);

    next_int!(u32, 4, next_be_u32, next_le_u32);
    next_int!(i32, 4, next_be_i32, next_le_i32);

    next_int!(u64, 8, next_be_u64, next_le_u64);
    next_int!(i64, 8, next_be_i64, next_le_i64);

    next_int!(u128, 16, next_be_u128, next_le_u128);
    next_int!(i128, 16, next_be_i128, next_le_i128);
}
