use crate::record_layer::{ContentType, ProtocolVersion, TLSRecord};
use anyhow::Result;
use std::io::{Read, Write};

/// The TLS version implemented.
pub const TLS_VERSION: ProtocolVersion = ProtocolVersion::new(1, 2);

pub fn do_handshake<C: Read + Write>(connection: &mut C) -> Result<()> {
    let data = vec![];
    let client_hello = TLSRecord::new(ContentType::Handshake, data);
    connection.write_all(&client_hello.as_bytes())?;
    Ok(())
}
