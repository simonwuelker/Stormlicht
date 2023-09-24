use crate::{
    alert::{Alert, AlertError},
    handshake::{ClientHello, HandshakeMessage},
    random::CryptographicRand,
    record_layer::{ContentType, TLSRecord},
};

use std::{
    io::{self, BufRead, BufReader, Read, Write},
    net::TcpStream,
};

#[derive(Debug)]
pub enum TLSError {
    UnknownContentType,
    InvalidTLSVersion,
    UnknownHandshakeMessageType,
    UnknownCipherSuite,
    UnknownCompressionMethod,
    Alert(AlertError),
    IO(io::Error),
}

/// The TLS version implemented.
pub const TLS_VERSION: ProtocolVersion = ProtocolVersion::new(1, 2);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProtocolVersion {
    pub major: u8,
    pub minor: u8,
}

impl ProtocolVersion {
    #[must_use]
    pub const fn new(major: u8, minor: u8) -> Self {
        Self { major, minor }
    }

    /// Note that the version is TLS 1.2 but the version number
    /// is slightly odd (`3.3`) since TLS 1.0 was the successor of OpenSSL 3.0
    /// which gave it the version number [0x03, 0x01] and so on.
    #[must_use]
    pub fn as_bytes(&self) -> [u8; 2] {
        [self.major + 2, self.minor + 1]
    }
}

impl TryFrom<[u8; 2]> for ProtocolVersion {
    type Error = TLSError;
    fn try_from(value: [u8; 2]) -> Result<Self, TLSError> {
        if value[0] < 3 || value[1] < 1 {
            log::warn!("Invalid TLS version: {}.{}", value[0], value[1]);
            Err(TLSError::InvalidTLSVersion)
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
    pub fn create(hostname: &str) -> Result<Self, TLSError> {
        let stream = TcpStream::connect((hostname, 443)).map_err(TLSError::IO)?;
        let mut connection = Self {
            writer: stream.try_clone().map_err(TLSError::IO)?,
            reader: BufReader::new(stream),
        };
        connection.do_handshake(hostname)?;

        Ok(connection)
    }

    pub fn do_handshake(&mut self, hostname: &str) -> Result<(), TLSError> {
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

        self.writer
            .write_all(&client_hello.into_bytes())
            .map_err(TLSError::IO)?;

        for _ in 0..10 {
            let response = self.next_tls_record()?;

            // TODO: fragmented messages are not yet supported
            match response.content_type() {
                ContentType::Alert => {
                    let alert = Alert::try_from(response.fragment()).map_err(TLSError::Alert)?;
                    dbg!(alert);
                },
                ContentType::Handshake => {
                    let handshake_msg = HandshakeMessage::new(response.fragment())?;
                    dbg!(handshake_msg);
                },
                _ => {},
            }
        }
        Ok(())
    }

    pub fn next_tls_record(&mut self) -> Result<TLSRecord, TLSError> {
        self.reader.fill_buf().map_err(TLSError::IO)?;

        let mut content_type_buffer = [0];
        self.reader
            .read_exact(&mut content_type_buffer)
            .map_err(TLSError::IO)?;
        let content_type = ContentType::try_from(content_type_buffer[0])?;

        let mut tls_version_buffer = [0, 0];
        self.reader
            .read_exact(&mut tls_version_buffer)
            .map_err(TLSError::IO)?;
        let tls_version = ProtocolVersion::try_from(tls_version_buffer)?;

        let mut length_buffer = [0, 0];
        self.reader
            .read_exact(&mut length_buffer)
            .map_err(TLSError::IO)?;
        let length = u16::from_be_bytes(length_buffer);

        let mut data = vec![0; length as usize];
        self.reader.read_exact(&mut data).map_err(TLSError::IO)?;

        Ok(TLSRecord::new(content_type, tls_version, length, data))
    }
}
