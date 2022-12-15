pub mod types;
pub mod wl_display;
pub mod wl_registry;

use types::*;
use wl_display::WaylandDisplayError;

use std::env;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::net::Shutdown;


#[derive(Debug)]
pub enum WaylandError {
    FailedToBindSocket(PathBuf),
    FailedToReadConfig,
    FailedToWriteToSocket,
    FailedToParseResponse,
    FailedToReadFromSocket,
    FailedToCloseConnection,
}

#[derive(Debug)]
pub struct WaylandHeader {
    pub object_id: u32,
    pub message_size: u16,
    pub opcode: u16,
}

impl WaylandHeader {
    pub fn new(object_id: u32, message_size: u16, opcode: u16) -> Self {
        Self {
            object_id: object_id,
            message_size: message_size,
            opcode: opcode
        }
    }
    pub fn as_bytes(self) -> [u8; 8] {
        let mut buffer = [0; 8];
        buffer[0..4].clone_from_slice(&self.object_id.to_ne_bytes());
        buffer[4..8].copy_from_slice(&((self.message_size as u32) << 16 | self.opcode as u32).to_ne_bytes());
        buffer
    }

    pub fn read<R: Read>(mut reader: R) -> Result<Self, WaylandError> {
        let mut buffer = [0; 8];
        reader.read_exact(&mut buffer).map_err(|_| WaylandError::FailedToReadFromSocket)?;
        let object_id = u32::from_ne_bytes(buffer[0..4].try_into().unwrap());
        let size_opcode = u32::from_ne_bytes(buffer[4..8].try_into().unwrap());

        Ok(Self {
            object_id: object_id,
            message_size: (size_opcode >> 16) as u16,
            opcode: (size_opcode & 0xFFFF) as u16,
        })
    }
}

fn load_wayland_config() -> Result<(), WaylandError> {
    let path = Path::new("/usr/share/wayland/wayland.xml");
    let mut config_file = File::open(path).map_err(|_| WaylandError::FailedToReadConfig)?;

    let mut contents = String::new();
    config_file.read_to_string(&mut contents).unwrap();

    // TODO parse config
    Ok(())
}


pub fn try_init() -> Result<Option<()>, WaylandError>{
    load_wayland_config()?;

    // Find wayland socket address
    let fd = match env::var("WAYLAND_SOCKET") {
        Ok(fd) => {PathBuf::from(fd)},
        _ => {
            match env::var("XDG_RUNTIME_DIR") {
                Ok(runtime_dir) => {
                    Path::new(&runtime_dir).join(&env::var("WAYLAND_DISPLAY").map_or(PathBuf::from("wayland-0"), |dir| PathBuf::from(dir)))
                },
                _ => {
                    // can't init wayland
                    return Ok(None);
                }
            }
        }
    };
    println!("Connecting to wayland server at {:?}", fd);

    // Bind wayland socket
    let mut wayland_stream = UnixStream::connect(&fd).map_err(|_| WaylandError::FailedToBindSocket(fd))?;

    // Bind registry interface
    let header = WaylandHeader::new(1, 12, 1);
    let body: [u8; 4] = wl_registry::WL_REGISTRY_ID.0.to_ne_bytes();
    wayland_stream.write_all(header.as_bytes().as_slice()).map_err(|_| WaylandError::FailedToWriteToSocket)?;
    wayland_stream.write_all(body.as_slice()).map_err(|_| WaylandError::FailedToWriteToSocket)?;

    loop {
        let response_header = WaylandHeader::read(&mut wayland_stream)?;
    
        match response_header.object_id {
            1 => wl_display::handle_response(response_header, &mut wayland_stream)?,
            2 => wl_registry::handle_response(response_header, &mut wayland_stream)?,
            _ => todo!(),
        }
    }
    

    // Close connection to the wayland compositor
    wayland_stream.shutdown(Shutdown::Both).map_err(|_| WaylandError::FailedToCloseConnection)?;
    Ok(Some(()))
}
