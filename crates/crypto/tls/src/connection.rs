use crate::{
    alert::{Alert, AlertError, Severity},
    encoding::{self, Cursor, Decoding},
    handshake::{self, ClientHello, Extension, HandshakeMessage},
    random::CryptographicRand,
    record_layer::{ContentType, TLSRecordReader, TLSRecordWriter},
    server_name::ServerName,
    Encoding,
};

use std::{
    io::{self, BufReader, Write},
    net::{self, TcpStream},
};

/// The TLS version implemented.
pub const TLS_VERSION: ProtocolVersion = ProtocolVersion::new(1, 2);

/// The destination port used for TLS connections
const TLS_PORT: u16 = 443;

impl Encoding for ProtocolVersion {
    fn encode(&self, bytes: &mut Vec<u8>) {
        bytes.extend_from_slice(&[self.major + 2, self.minor + 1])
    }
}

impl<'a> Decoding<'a> for ProtocolVersion {
    fn decode(cursor: &mut Cursor<'a>) -> encoding::Result<Self> {
        let buf: [u8; 2] = cursor.decode()?;

        if buf[0] < 2 || buf[1] == 0 {
            log::warn!("Invalid TLS version: {}.{}", buf[0], buf[1]);
            return Err(encoding::Error);
        }

        Ok(Self::new(buf[0] - 2, buf[1] - 1))
    }
}

#[derive(Debug)]
pub enum TLSError {
    BadMessage,
    FatalAlert,
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
        let ip = net::IpAddr::try_from(&server_name)?;
        let stream = TcpStream::connect((ip, TLS_PORT))?;
        let writer = TLSRecordWriter::new(stream.try_clone()?);
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
        let mut client_hello = ClientHello {
            client_random,
            extensions: vec![
                Extension::StatusRequest,
                Extension::RenegotiationInfo,
                Extension::SignedCertificateTimestamp,
            ],
        };

        if let ServerName::Domain(domain) = server_name {
            // If we have a domain name we can use the SNI extension
            client_hello
                .extensions
                .push(handshake::Extension::ServerName(domain))
        }

        let mut client_hello_writer = self.writer.writer_for(ContentType::Handshake)?;
        client_hello_writer.write_all(&client_hello.as_bytes())?;
        client_hello_writer.flush()?;

        for _ in 0..10 {
            let record = self.reader.next_record()?;

            // TODO: fragmented messages are not yet supported
            match record.content_type {
                ContentType::Alert => {
                    let alert = Alert::try_from(record.data.as_slice())?;
                    match alert.severity {
                        Severity::Fatal => {
                            log::error!("Fatal Alert: {:?}", alert.description);
                            return Err(TLSError::FatalAlert);
                        },
                        Severity::Warning => {
                            log::warn!("Alert: {:?}", alert.description);
                        },
                    }
                },
                ContentType::Handshake => {
                    let _handshake_msg = HandshakeMessage::new(&record.data)?;
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

impl From<io::Error> for TLSError {
    fn from(value: io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<dns::DNSError> for TLSError {
    fn from(value: dns::DNSError) -> Self {
        Self::DNS(value)
    }
}

impl From<AlertError> for TLSError {
    fn from(value: AlertError) -> Self {
        Self::Alert(value)
    }
}
