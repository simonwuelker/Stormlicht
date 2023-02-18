use crate::{
    alert::Alert,
    handshake::{ClientHello, HandshakeType},
    random::CryptographicRand,
    record_layer::{ContentType, TLSRecord},
};
use anyhow::{Context, Result};
use std::{
    io::{BufRead, BufReader, Read, Write},
    net::TcpStream,
};
use thiserror::Error;

#[derive(Clone, Copy, Debug, Error)]
pub enum TLSError {
    #[error("Unknown TLS record content type: {}", .0)]
    UnknownContentType(u8),
    #[error("Unknown TLS version {}.{}", .0, .1)]
    InvalidTLSVersion(u8, u8),
    #[error("Unknown handshake message type: {}", .0)]
    UnknownHandshakeMessageType(u8),
}

/// The TLS version implemented.
pub const TLS_VERSION: ProtocolVersion = ProtocolVersion::new(1, 2);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProtocolVersion {
    pub major: u8,
    pub minor: u8,
}

impl ProtocolVersion {
    pub const fn new(major: u8, minor: u8) -> Self {
        Self { major, minor }
    }

    /// Note that the version is TLS 1.2 but the version number
    /// is slightly odd (`3.3`) since TLS 1.0 was the successor of OpenSSL 3.0
    /// which gave it the version number [0x03, 0x01] and so on.
    pub fn as_bytes(&self) -> [u8; 2] {
        [self.major + 2, self.minor + 1]
    }
}

impl TryFrom<[u8; 2]> for ProtocolVersion {
    type Error = TLSError;
    fn try_from(value: [u8; 2]) -> Result<Self, TLSError> {
        if value[0] < 3 || value[1] < 1 {
            Err(TLSError::InvalidTLSVersion(value[0], value[1]))
        } else {
            Ok(Self::new(value[0] - 2, value[1] - 1))
        }
    }
}

#[derive(Debug)]
pub struct TLSConnection {
    writer: TcpStream,
    reader: BufReader<TcpStream>,
}

impl TLSConnection {
    pub fn create(hostname: &str) -> Result<Self> {
        let stream = TcpStream::connect((hostname, 443))?;
        let mut connection = Self {
            writer: stream.try_clone()?,
            reader: BufReader::new(stream),
        };
        connection
            .do_handshake(hostname)
            .with_context(|| format!("Failed to perform handshake with {}", hostname))?;

        Ok(connection)
    }

    pub fn do_handshake(&mut self, hostname: &str) -> Result<()> {
        let mut client_random = [0; 32];
        let mut rng = CryptographicRand::new().unwrap();
        client_random[0..16].copy_from_slice(&rng.next_u128().to_ne_bytes());
        client_random[16..32].copy_from_slice(&rng.next_u128().to_ne_bytes());

        // NOTE: We override the version here because some TLS server apparently fail if the version isn't 1.0
        // for the initial ClientHello
        // This is also mentioned in https://www.rfc-editor.org/rfc/rfc5246#appendix-E
        let client_hello = TLSRecord::from_data_and_version(
            ContentType::Handshake,
            ProtocolVersion::new(1, 0),
            ClientHello::new(hostname, client_random).into_bytes(),
        );
        self.writer.write_all(&client_hello.into_bytes())?;

        let response = self
            .next_tls_record()
            .context("Failed to read TLS record")?;

        match response.content_type() {
            ContentType::Alert => {
                let alert = Alert::try_from(response.fragment())?;
                dbg!(alert);
            },
            ContentType::Handshake => {
                for i in 0..response.fragment().len() {
                    if i % 32 == 0 {
                        println!()
                    }
                    print!("{:0>2x} ", response.fragment()[i]);
                }
                let fragment = response.fragment();
                let handshake_type = HandshakeType::try_from(fragment[0])
                    .map_err(TLSError::UnknownHandshakeMessageType)?;
                dbg!(handshake_type);

                let mut length_bytes = [0, 0, 0, 0];
                length_bytes[1..].copy_from_slice(&fragment[1..4]);
                let message_length = u32::from_be_bytes(length_bytes) as usize;

                if fragment.len() - 4 != message_length {
                    todo!(
                        "Message is fragmented (message len {message_length} but we only got {}",
                        fragment.len() - 4
                    );
                }
            },
            _ => {},
        }
        Ok(())
    }

    pub fn next_tls_record(&mut self) -> Result<TLSRecord> {
        self.reader.fill_buf()?;

        let mut content_type_buffer = [0];
        self.reader.read_exact(&mut content_type_buffer)?;
        let content_type =
            ContentType::try_from(content_type_buffer[0]).map_err(TLSError::UnknownContentType)?;

        let mut tls_version_buffer = [0, 0];
        self.reader.read_exact(&mut tls_version_buffer)?;
        let tls_version = ProtocolVersion::try_from(tls_version_buffer)?;

        let mut length_buffer = [0, 0];
        self.reader.read_exact(&mut length_buffer)?;
        let length = u16::from_be_bytes(length_buffer);

        let mut data = vec![0; length as usize];
        self.reader.read_exact(&mut data)?;

        Ok(TLSRecord::new(content_type, tls_version, length, data))
    }
}
