use crate::{
    alert::{Alert, AlertError},
    handshake::{self, ClientHello, HandshakeMessage},
    random::CryptographicRand,
    record_layer::{ContentType, TLSRecordReader, TLSRecordWriter},
    server_name::ServerName,
};

use std::{
    io::{self, BufReader},
    net::{self, TcpStream},
};

/// The TLS version implemented.
pub const TLS_VERSION: ProtocolVersion = ProtocolVersion::new(1, 2);

/// The destination port used for TLS connections
const TLS_PORT: u16 = 443;

#[derive(Debug)]
pub enum TLSError {
    UnknownContentType,
    InvalidTLSVersion,
    UnsupportedTLSVersion,
    UnknownHandshakeMessageType,
    UnknownCipherSuite,
    UnknownCompressionMethod,
    Alert(AlertError),
    DNS(dns::DNSError),
    IO(io::Error),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
    pub const fn as_bytes(&self) -> [u8; 2] {
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
    writer: TLSRecordWriter<TcpStream>,
    reader: TLSRecordReader<BufReader<TcpStream>>,
}

impl TLSConnection {
    pub fn establish<A>(addr: A) -> Result<Self, TLSError>
    where
        ServerName: From<A>,
    {
        let server_name = ServerName::from(addr);
        let ip = net::IpAddr::try_from(&server_name).map_err(TLSError::DNS)?;
        let stream = TcpStream::connect((ip, TLS_PORT)).map_err(TLSError::IO)?;
        let writer = TLSRecordWriter::new(stream.try_clone().map_err(TLSError::IO)?);
        let reader = TLSRecordReader::new(BufReader::new(stream));
        let mut connection = Self { writer, reader };

        connection.do_handshake(server_name)?;

        Ok(connection)
    }

    pub fn do_handshake(&mut self, server_name: ServerName) -> Result<(), TLSError> {
        let mut client_random = [0; 32];
        let mut rng = CryptographicRand::new().unwrap();
        client_random[0..16].copy_from_slice(&rng.next_u128().to_ne_bytes());
        client_random[16..32].copy_from_slice(&rng.next_u128().to_ne_bytes());

        // NOTE: We override the version here because some TLS server apparently fail if the version isn't 1.0
        // for the initial ClientHello
        // This is also mentioned in https://www.rfc-editor.org/rfc/rfc5246#appendix-E
        let mut client_hello = ClientHello::new(client_random);

        if let ServerName::Domain(domain) = server_name {
            // If we have a domain name we can use the SNI extension
            client_hello.add_extension(handshake::Extension::ServerName(domain))
        }

        let client_hello_writer = self
            .writer
            .writer_for(ContentType::Handshake)
            .map_err(TLSError::IO)?;
        client_hello
            .write_to(client_hello_writer)
            .map_err(TLSError::IO)?;

        for _ in 0..10 {
            let record = self.reader.next_record()?;

            // TODO: fragmented messages are not yet supported
            match record.content_type {
                ContentType::Alert => {
                    let alert = Alert::try_from(record.data.as_slice()).map_err(TLSError::Alert)?;
                    dbg!(alert);
                },
                ContentType::Handshake => {
                    let handshake_msg = HandshakeMessage::new(&record.data)?;
                    dbg!(handshake_msg);
                },
                _ => {},
            }
        }
        Ok(())
    }
}

impl io::Read for TLSConnection {
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        todo!()
    }
}

impl io::Write for TLSConnection {
    fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
        todo!()
    }

    fn flush(&mut self) -> io::Result<()> {
        todo!()
    }
}
