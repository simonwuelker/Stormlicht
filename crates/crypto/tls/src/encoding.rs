//! Provides memory-bound, infallible IO
//!
//! Using the regular `std::io` often doesn't make sense here, as we are almost
//! always working on in-memory buffers anyways. (In which case the only possible
//! error is an OOM, which we aren't handling)

use std::marker::PhantomData;

use crate::TLSError;

#[derive(Clone, Copy)]
pub struct Cursor<'a> {
    bytes: &'a [u8],
    cursor: usize,
}

impl<'a> Cursor<'a> {
    #[must_use]
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, cursor: 0 }
    }

    #[must_use]
    pub fn remainder(&self) -> &[u8] {
        &self.bytes[self.cursor..]
    }

    pub fn decode<T>(&mut self) -> Result<T>
    where
        T: Decoding<'a>,
    {
        T::decode(self)
    }

    pub fn advance(&mut self, advance_by: usize) {
        self.cursor += advance_by;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Error;

pub type Result<T> = std::result::Result<T, Error>;

pub trait Encoding {
    /// Serialize the object as bytes into the provided vector
    ///
    /// The reason we don't use a generic `io::Write` instance here is that in TLS,
    /// we almost always need the length of the message *first*, so we would need
    /// to construct a intermediate vector first anyways, at which point `io::Write`
    /// just becomes a pointless abstraction.
    fn encode(&self, bytes: &mut Vec<u8>);

    fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        self.encode(&mut bytes);
        bytes
    }
}

pub trait Decoding<'a>: Sized {
    fn decode(cursor: &mut Cursor<'a>) -> Result<Self>;
}

impl<const N: usize> Encoding for [u8; N] {
    fn encode(&self, bytes: &mut Vec<u8>) {
        bytes.extend_from_slice(self.as_slice())
    }
}

impl<'a, const N: usize> Decoding<'a> for [u8; N] {
    fn decode(cursor: &mut Cursor<'_>) -> Result<Self> {
        let remainder = cursor.remainder();
        if remainder.len() < N {
            return Err(Error);
        }

        let buf = remainder[..N]
            .try_into()
            .expect("Slice of size N can always be cast to array of size N");
        cursor.advance(N);
        Ok(buf)
    }
}

macro_rules! encoding_for_number {
    ($number: ty) => {
        impl Encoding for $number {
            fn encode(&self, bytes: &mut Vec<u8>) {
                bytes.extend_from_slice(self.to_be_bytes().as_slice())
            }
        }

        impl<'a> Decoding<'a> for $number {
            fn decode(cursor: &mut Cursor<'a>) -> Result<Self> {
                const N: usize = std::mem::size_of::<$number>();
                let buf: [u8; N] = cursor.decode()?;
                Ok(Self::from_be_bytes(buf))
            }
        }
    };
}

encoding_for_number!(u8);
encoding_for_number!(u16);
encoding_for_number!(u32);
encoding_for_number!(u64);
encoding_for_number!(u128);

#[derive(Clone, Copy)]
pub struct WithLengthPrefix<'a, L, T>
where
    T: ?Sized,
{
    element: &'a T,
    marker: PhantomData<L>,
}

impl<'a, L, T> WithLengthPrefix<'a, L, T>
where
    T: Encoding + ?Sized,
{
    #[must_use]
    pub const fn new(element: &'a T) -> Self {
        Self {
            element,
            marker: PhantomData,
        }
    }
}

macro_rules! lp_buffer_with_length {
    ($num: ty) => {
        impl<'a, T> Encoding for WithLengthPrefix<'a, $num, T>
        where
            T: Encoding + ?Sized,
        {
            fn encode(&self, bytes: &mut Vec<u8>) {
                let content_bytes = self.element.as_bytes();

                let length = <$num>::try_from(content_bytes.len()).expect("Buffer too long");
                length.encode(bytes);
                bytes.extend(content_bytes);
            }
        }
    };
}

lp_buffer_with_length!(u8);
lp_buffer_with_length!(u16);

pub type WithU8LengthPrefix<'a, T> = WithLengthPrefix<'a, u8, T>;
pub type WithU16LengthPrefix<'a, T> = WithLengthPrefix<'a, u16, T>;

impl<T> Encoding for [T]
where
    T: Encoding,
{
    fn encode(&self, bytes: &mut Vec<u8>) {
        for element in self {
            element.encode(bytes);
        }
    }
}

#[macro_export]
macro_rules! enum_encoding {
    (
        $(#[$doccomments:meta])*
        $visibility:vis enum $name:ident($size: ty)
        { $( $variant: ident = $value: expr,)* }
    ) => {
        $(#[$doccomments])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        $visibility enum $name {
            $($variant),*
        }

        impl $crate::encoding::Encoding for $name {
            fn encode(&self, bytes: &mut Vec<u8>) {
                match self {
                    $(Self::$variant => ($value as $size).encode(bytes),)*
                }
            }
        }

        impl<'a> $crate::encoding::Decoding<'a> for $name {
            fn decode(cursor: &mut $crate::encoding::Cursor<'a>) -> $crate::encoding::Result<Self> {
                let n: $size = cursor.decode()?;
                let enum_value = match n {
                    $($value => Self::$variant,)*
                    _ => return Err($crate::encoding::Error),
                };
                Ok(enum_value)
            }
        }
    };
}

/// 24-bit unsigned integer
#[derive(Clone, Copy, Debug)]
pub struct U24(u32);

impl Encoding for U24 {
    fn encode(&self, bytes: &mut Vec<u8>) {
        bytes.extend_from_slice(&self.0.to_be_bytes()[1..])
    }
}

impl<'a> Decoding<'a> for U24 {
    fn decode(cursor: &mut Cursor<'a>) -> Result<Self> {
        let remainder = cursor.remainder();
        if remainder.len() < 3 {
            return Err(Error);
        }

        let mut buf = [0; 4];
        buf[1..].copy_from_slice(&remainder[..3]);
        cursor.advance(3);
        Ok(Self(u32::from_be_bytes(buf)))
    }
}

impl From<U24> for usize {
    fn from(value: U24) -> Self {
        value.0 as usize
    }
}

impl From<Error> for TLSError {
    fn from(_: Error) -> Self {
        TLSError::BadMessage
    }
}
