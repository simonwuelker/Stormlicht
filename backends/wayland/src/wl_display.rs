use super::{
    types::{WaylandObjectID, WaylandString, WaylandType, WaylandUInt},
    WaylandError, WaylandHeader,
};
use std::io::Read;

pub const WL_DISPLAY_ID: WaylandObjectID = WaylandObjectID(1);

#[derive(Debug)]
pub enum WaylandDisplayError {
    /// Server couldn't find object
    InvalidObject,
    /// Method doesn't exist on the specified interface or malformed request
    InvalidMethod,
    /// Server is out of memory
    OutOfMemory,
    /// Implementation error in compositor
    Implementation,
}

impl TryFrom<WaylandUInt> for WaylandDisplayError {
    type Error = ();

    fn try_from(from: WaylandUInt) -> Result<Self, Self::Error> {
        match from.0 {
            0 => Ok(Self::InvalidObject),
            1 => Ok(Self::InvalidMethod),
            2 => Ok(Self::OutOfMemory),
            3 => Ok(Self::Implementation),
            _ => Err(()),
        }
    }
}

pub fn handle_response<R: Read>(header: WaylandHeader, mut reader: R) -> Result<(), WaylandError> {
    match header.opcode {
        0 => {
            // Error
            let _object_id = WaylandObjectID::read(&mut reader)?;
            let _error = WaylandDisplayError::try_from(WaylandUInt::read(&mut reader)?)
                .map_err(|_| WaylandError::FailedToParseResponse)?;
            let string = WaylandString::read(&mut reader)?;
            println!("Error: {string:?}");
        },
        _ => todo!(),
    }
    Ok(())
}
