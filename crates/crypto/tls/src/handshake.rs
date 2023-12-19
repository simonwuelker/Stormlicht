use crate::{
    certificate::{self, SignedCertificate, X509Certificate},
    connection::{ProtocolVersion, TLSError, TLS_VERSION},
    der::Deserialize,
    encoding::{Cursor, Decoding, WithU16LengthPrefix, WithU8LengthPrefix, U24},
    enum_encoding, CipherSuite, Encoding, SessionId,
};

/// A list of all cipher suites supported by this implementation
const SUPPORTED_CIPHER_SUITES: [CipherSuite; 1] = [CipherSuite::TlsRsaWithAes128CbcSha];

enum_encoding!(
    /// TLS Compression methods are defined in [RFC 3749](https://www.rfc-editor.org/rfc/rfc3749)
    ///
    /// # Security
    /// Encrypting compressed data can compromise security.
    /// See [CRIME](https://en.wikipedia.org/wiki/CRIME) and [BREACH](https://en.wikipedia.org/wiki/BREACH)
    /// for more information.
    ///
    /// We will therefore **never** set a [CompressionMethod] other than [CompressionMethod::None].
    /// Seeing how future TLS protocol version removed this option altogether, this
    /// seems like the correct approach.
    pub enum CompressionMethod(u8) {
        None = 0x00,
        Deflate = 0x01,
    }
);

impl TryFrom<u8> for CompressionMethod {
    type Error = TLSError;

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Deflate),
            other => {
                log::warn!("Unknown TLS compression method: {other}");
                Err(TLSError::UnknownCompressionMethod)
            },
        }
    }
}

enum_encoding! {
    pub enum HandshakeType(u8) {
        HelloRequest = 0,
        ClientHello = 1,
        ServerHello = 2,
        Certificate = 11,
        ServerKeyExchange = 12,
        CertificateRequest = 13,
        ServerHelloDone = 14,
        CertificateVerify = 15,
        ClientKeyExchange = 16,
        Finished = 20,
        CertificateStatus = 22,
    }
}

#[derive(Clone, Debug)]
pub struct ClientHello {
    pub client_random: [u8; 32],
    pub extensions: Vec<Extension>,
}

#[derive(Clone, Debug)]
pub struct ServerHello {
    pub version: ProtocolVersion,
    pub server_random: [u8; 32],
    pub session_id: SessionId,
    pub selected_cipher_suite: CipherSuite,
    pub selected_compression_method: CompressionMethod,
    pub extensions: Vec<Extension>,
}

#[derive(Clone, Debug)]
pub enum HandshakeMessage {
    ClientHello(ClientHello),
    ServerHello(ServerHello),
    Certificate(CertificateChain),
    CertificateStatus,
    ServerHelloDone,
}

#[derive(Clone, Debug)]
pub enum CertificateChain {
    X509v3(Vec<X509Certificate>),
}

impl Encoding for ClientHello {
    fn encode(&self, bytes: &mut Vec<u8>) {
        HandshakeType::ClientHello.encode(bytes);

        // The length of the clienthello message, will be patched later
        let offset = bytes.len();
        bytes.extend_from_slice(&[0xAB, 0xAB, 0xAB]);

        TLS_VERSION.encode(bytes);

        self.client_random.encode(bytes);

        bytes.push(0x00); // No session id to resume

        // Supported Cipher Suites
        WithU16LengthPrefix::new(SUPPORTED_CIPHER_SUITES.as_slice()).encode(bytes);

        // Compression method
        // Since compression can compromise security (see CRIME), we will
        // never use compression.
        WithU8LengthPrefix::new([CompressionMethod::None].as_slice()).encode(bytes);

        // Extensions
        WithU16LengthPrefix::new(self.extensions.as_slice()).encode(bytes);

        // Patch the length of the message
        let clienthello_length = (bytes.len() - offset) as u32 - 3;
        bytes[offset..offset + 3].copy_from_slice(&clienthello_length.to_be_bytes()[1..]);
    }
}

#[derive(Clone, Debug)]
pub enum Extension {
    ServerName(String),
    StatusRequest,
    RenegotiationInfo,
    SignedCertificateTimestamp,
}

impl HandshakeMessage {
    pub fn new(message_data: &[u8]) -> Result<Self, TLSError> {
        // Every Handshake message starts with the same header
        // * 1 bytes message type
        // * 3 bytes length
        // Everything after that depends on the message type
        if message_data.len() < 4 {
            todo!("fragmented message");
        }
        let mut header = Cursor::new(message_data);
        let handshake_type = header.decode()?;
        let length: U24 = header.decode()?;

        let mut message_buf = Vec::with_capacity(length.into());
        message_buf.extend_from_slice(header.remainder());

        // FIXME: Fully assemble fragmented message
        if message_data.len() - 4 != length.into() {
            todo!(
                "Message is fragmented (message len {:?} but we only got {}",
                length,
                message_data.len() - 4
            );
        };

        let mut message = Cursor::new(&message_data[4..]);

        match handshake_type {
            HandshakeType::ServerHello => {
                // https://www.rfc-editor.org/rfc/rfc5246#section-7.4.1.3
                let server_version = ProtocolVersion::decode(&mut message)?; // message.decode()?;
                let server_random: [u8; 32] = message.decode()?;
                let session_id = message.decode()?;
                let selected_cipher_suite = message.decode()?;
                let selected_compression_method = message.decode()?;

                let server_hello = Self::ServerHello(ServerHello {
                    version: server_version,
                    server_random,
                    session_id,
                    selected_cipher_suite,
                    selected_compression_method,
                    extensions: vec![],
                });
                Ok(server_hello)
            },
            HandshakeType::Certificate => {
                // https://www.rfc-editor.org/rfc/rfc5246#section-7.4.2
                let certificate_chain_length: usize = message.decode::<U24>()?.into();

                let mut certificate_chain = vec![];

                let mut bytes_read: usize = 0;
                while bytes_read != certificate_chain_length {
                    let certificate_length: usize = message.decode::<U24>()?.into();

                    let remainder = message.remainder();
                    if remainder.len() < certificate_length {
                        return Err(TLSError::BadMessage);
                    }

                    // FIXME: propagate error
                    let signed_cert = SignedCertificate::from_bytes(
                        &remainder[..certificate_length],
                        certificate::Error::TrailingBytes,
                    )
                    .expect("certificate parsing failed");

                    if !signed_cert.is_valid() {
                        log::warn!("Browser supplied invalid certificate");
                    }

                    message.advance(certificate_length);
                    certificate_chain.push(signed_cert.into());
                    bytes_read += certificate_length + 3;
                }

                Ok(Self::Certificate(CertificateChain::X509v3(
                    certificate_chain,
                )))
            },
            HandshakeType::CertificateStatus => {
                // FIXME: Handle this
                Ok(Self::CertificateStatus)
            },
            HandshakeType::ServerHelloDone => Ok(Self::ServerHelloDone),
            _ => unimplemented!("Parse {handshake_type:?} message"),
        }
    }
}

impl Encoding for Extension {
    fn encode(&self, bytes: &mut Vec<u8>) {
        bytes.extend_from_slice(&self.into_bytes())
    }
}

impl Extension {
    pub fn length(&self) -> usize {
        match self {
            Self::ServerName(hostname) => {
                2 // Extension identifier
                 + 2 // Extension length
                 + 2 // First list entry length
                 + 1 // Entry identifier
                  + 2 // hostname length
                  + hostname.len() // hostname
            },
            Self::StatusRequest => 9,
            Self::RenegotiationInfo => 5,
            Self::SignedCertificateTimestamp => 4,
        }
    }

    pub fn into_bytes(&self) -> Vec<u8> {
        match self {
            Self::ServerName(hostname) => {
                let hostname_bytes = hostname.as_bytes();
                let mut extension_data = Vec::with_capacity(9 + hostname_bytes.len());
                let hostname_len = hostname_bytes.len() as u16;

                // Assigned value for server name extension
                extension_data.extend_from_slice(&[0x00, 0x00]);

                // Server name extension length
                extension_data.extend_from_slice(&(5 + hostname_len).to_be_bytes());

                // First (and only) list entry length
                extension_data.extend_from_slice(&(3 + hostname_len).to_be_bytes());

                // Entry is a DNS hostname
                extension_data.push(0x00);

                // hostname length
                extension_data.extend_from_slice(&hostname_len.to_be_bytes());

                // The actual hostname
                extension_data.extend_from_slice(hostname_bytes);

                extension_data
            },
            Self::StatusRequest => {
                let mut extension_data = Vec::with_capacity(9);

                // Assigned value for status request extension
                extension_data.extend_from_slice(&[0x00, 0x05]);

                // Status request extension length
                extension_data.extend_from_slice(&5u16.to_be_bytes());

                // OCSP status type
                extension_data.push(0x01);

                // No responder ID
                extension_data.extend_from_slice(&[0x00, 0x00]);

                // No request extension information
                extension_data.extend_from_slice(&[0x00, 0x00]);

                extension_data
            },
            Self::RenegotiationInfo => {
                let mut extension_data = Vec::with_capacity(5);

                // Assigned value for renegotiation info extension
                extension_data.extend_from_slice(&[0xFF, 0x01]);

                // Status request extension length
                extension_data.extend_from_slice(&1u16.to_be_bytes());

                extension_data.push(0x00); // new connection

                extension_data
            },
            Self::SignedCertificateTimestamp => {
                let mut extension_data = Vec::with_capacity(4);

                // Assigned value for renegotiation info extension
                extension_data.extend_from_slice(&[0x00, 0x12]);

                // Status request extension length
                extension_data.extend_from_slice(&0u16.to_be_bytes());

                extension_data
            },
        }
    }
}
