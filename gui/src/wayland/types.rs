//! Wayland specific data types

use std::io::Read;
use super::WaylandError;

/// A string, prefixed with a 32-bit integer specifying its length (in bytes), 
/// followed by the string contents and a NUL terminator, padded to 32 bits with undefined data. 
/// The encoding is not specified, but in practice UTF-8 is used.
#[derive(Debug, PartialEq)]
pub struct WaylandString(String);

#[derive(Debug, PartialEq)]
pub struct WaylandObjectID(pub u32);

#[derive(Debug, PartialEq)]
pub struct WaylandInt(pub i32);

#[derive(Debug, PartialEq)]
pub struct WaylandUInt(pub u32);

#[derive(Debug, PartialEq)]
pub struct WaylandFixed(u32);

pub trait WaylandType: std::fmt::Debug + Sized {
    fn read<R: Read>(reader: R) -> Result<Self, WaylandError>;
}

impl WaylandType for WaylandString {
    fn read<R: Read>(mut reader: R) -> Result<Self, WaylandError> {
        let mut length_buffer = [0; 4];
        reader.read_exact(&mut length_buffer).map_err(|_| WaylandError::FailedToParseResponse)?;
        let mut length = u32::from_ne_bytes(length_buffer) as usize;

        // length is always rounded to the next multiple of 4
        if length % 4 != 0 {
            length += 4 - (length % 4);
        }

        let mut string_bytes = vec![0; length];
        reader.read_exact(&mut string_bytes).map_err(|_| WaylandError::FailedToParseResponse)?;

        let null_byte_at = string_bytes.iter().position(|b| *b == 0).ok_or(WaylandError::FailedToParseResponse)?;
        string_bytes.truncate(null_byte_at);
        let string = String::from_utf8(string_bytes).map_err(|_| WaylandError::FailedToParseResponse)?;
        Ok(Self(string))
    }
}

impl WaylandType for WaylandObjectID {
    fn read<R: Read>(mut reader: R) -> Result<Self, WaylandError> {
        let mut value_buffer = [0; 4];
        reader.read_exact(&mut value_buffer).map_err(|_| WaylandError::FailedToParseResponse)?;
        let value = u32::from_ne_bytes(value_buffer);
        Ok(Self(value))
    }
}

impl WaylandType for WaylandInt {
    fn read<R: Read>(mut reader: R) -> Result<Self, WaylandError> {
        let mut value_buffer = [0; 4];
        reader.read_exact(&mut value_buffer).map_err(|_| WaylandError::FailedToParseResponse)?;
        let value = i32::from_ne_bytes(value_buffer);
        Ok(Self(value))
    }
}

impl WaylandType for WaylandUInt {
    fn read<R: Read>(mut reader: R) -> Result<Self, WaylandError> {
        let mut value_buffer = [0; 4];
        reader.read_exact(&mut value_buffer).map_err(|_| WaylandError::FailedToParseResponse)?;
        let value = u32::from_ne_bytes(value_buffer);
        Ok(Self(value))
    }
}