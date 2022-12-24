use std::io::Read;

use crate::wayland::types::{WaylandString, WaylandType};

use super::{
    types::{WaylandObjectID, WaylandUInt},
    WaylandError, WaylandHeader,
};
pub const WL_REGISTRY_ID: WaylandObjectID = WaylandObjectID(2);

pub fn handle_response<R: Read>(header: WaylandHeader, mut reader: R) -> Result<(), WaylandError> {
    match header.opcode {
        0 => {
            // announce global
            let _name = WaylandUInt::read(&mut reader)?;
            let interface = WaylandString::read(&mut reader)?;
            let _version = WaylandUInt::read(&mut reader)?;
            println!("announce global: {interface:?}");
        },
        _ => todo!(),
    }
    Ok(())
}
