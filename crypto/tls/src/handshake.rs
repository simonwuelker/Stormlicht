use std::io::{Cursor, Read};

use crate::{
    certificate::X509v3Certificate,
    connection::{ProtocolVersion, TLSError, TLS_VERSION},
    CipherSuite,
};

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
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CompressionMethod {
    None,
    Deflate,
}

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

#[derive(Clone, Copy, Debug)]
pub enum HandshakeType {
    HelloRequest,
    ClientHello,
    ServerHello,
    Certificate,
    ServerKeyExchange,
    CertificateRequest,
    ServerHelloDone,
    CertificateVerify,
    ClientKeyExchange,
    Finished,
}

impl From<HandshakeType> for u8 {
    fn from(value: HandshakeType) -> Self {
        match value {
            HandshakeType::HelloRequest => 0,
            HandshakeType::ClientHello => 1,
            HandshakeType::ServerHello => 2,
            HandshakeType::Certificate => 11,
            HandshakeType::ServerKeyExchange => 12,
            HandshakeType::CertificateRequest => 13,
            HandshakeType::ServerHelloDone => 14,
            HandshakeType::CertificateVerify => 15,
            HandshakeType::ClientKeyExchange => 16,
            HandshakeType::Finished => 20,
        }
    }
}

impl TryFrom<u8> for HandshakeType {
    type Error = TLSError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::HelloRequest),
            1 => Ok(Self::ClientHello),
            2 => Ok(Self::ServerHello),
            11 => Ok(Self::Certificate),
            12 => Ok(Self::ServerKeyExchange),
            13 => Ok(Self::CertificateRequest),
            14 => Ok(Self::ServerHelloDone),
            15 => Ok(Self::CertificateVerify),
            16 => Ok(Self::ClientKeyExchange),
            20 => Ok(Self::Finished),
            other => {
                log::warn!("Unknown TLS handshake message type: {other}");
                Err(TLSError::UnknownHandshakeMessageType)
            },
        }
    }
}
#[derive(Clone, Debug)]
pub struct ClientHello {
    version: ProtocolVersion,
    client_random: [u8; 32],
    session_id_to_resume: Option<Vec<u8>>,
    supported_cipher_suites: Vec<CipherSuite>,
    extensions: Vec<Extension>,
}

#[derive(Clone, Debug)]
pub struct ServerHello {
    version: ProtocolVersion,
    server_random: [u8; 32],
    session_id: Option<Vec<u8>>,
    selected_cipher_suite: CipherSuite,
    selected_compression_method: CompressionMethod,
    extensions: Vec<Extension>,
}

#[derive(Clone, Debug)]
pub enum HandshakeMessage {
    ClientHello(ClientHello),
    ServerHello(ServerHello),
    Certificate(CertificateChain),
    ServerHelloDone,
}

#[derive(Clone, Debug)]
pub enum CertificateChain {
    X509v3(Vec<X509v3Certificate>),
}

impl ServerHello {
    pub fn version(&self) -> ProtocolVersion {
        self.version
    }

    pub fn server_random(&self) -> [u8; 32] {
        self.server_random
    }

    pub fn session_id(&self) -> &Option<Vec<u8>> {
        &self.session_id
    }

    pub fn selected_cipher_suite(&self) -> CipherSuite {
        self.selected_cipher_suite
    }

    pub fn selected_compression_method(&self) -> CompressionMethod {
        self.selected_compression_method
    }

    pub fn extensions(&self) -> &[Extension] {
        &self.extensions
    }
}

impl ClientHello {
    pub fn new(hostname: &str, client_random: [u8; 32]) -> Self {
        Self {
            version: TLS_VERSION,
            client_random: client_random,
            session_id_to_resume: None,
            supported_cipher_suites: vec![CipherSuite::TLS_RSA_WITH_AES_128_CBC_SHA],
            extensions: vec![
                Extension::ServerName(hostname.to_string()),
                Extension::StatusRequest,
                Extension::RenegotiationInfo,
                Extension::SignedCertificateTimestamp,
            ],
        }
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

        let handshake_type = HandshakeType::try_from(message_data[0])?;

        let mut length_bytes = [0; 4];
        length_bytes[1..].copy_from_slice(&message_data[1..4]);
        let message_length = u32::from_be_bytes(length_bytes) as usize;

        if message_data.len() - 4 != message_length {
            todo!(
                "Message is fragmented (message len {message_length} but we only got {}",
                message_data.len() - 4
            );
        };

        let mut reader = Cursor::new(&message_data[4..]);

        match handshake_type {
            HandshakeType::ServerHello => {
                let mut server_version_bytes: [u8; 2] = [0, 0];
                reader
                    .read_exact(&mut server_version_bytes)
                    .map_err(TLSError::IO)?;
                let server_version = ProtocolVersion::try_from(server_version_bytes)?;

                let mut server_random: [u8; 32] = [0; 32];
                reader
                    .read_exact(&mut server_random)
                    .map_err(TLSError::IO)?;

                let mut session_id_length_buffer = [0];
                reader
                    .read_exact(&mut session_id_length_buffer)
                    .map_err(TLSError::IO)?;
                let session_id_length = session_id_length_buffer[0];

                let session_id = if session_id_length == 0 {
                    None
                } else {
                    let mut session_id = vec![0; session_id_length as usize];
                    reader.read_exact(&mut session_id).map_err(TLSError::IO)?;
                    Some(session_id)
                };

                let mut cipher_suite_bytes = [0, 0];
                reader
                    .read_exact(&mut cipher_suite_bytes)
                    .map_err(TLSError::IO)?;
                let cipher_suite = CipherSuite::try_from(cipher_suite_bytes)?;

                let mut selected_compression_method_buffer = [0];
                reader
                    .read_exact(&mut selected_compression_method_buffer)
                    .map_err(TLSError::IO)?;
                let selected_compression_method =
                    CompressionMethod::try_from(selected_compression_method_buffer[0])?;

                let server_hello = Self::ServerHello(ServerHello {
                    version: server_version,
                    server_random: server_random,
                    session_id: session_id,
                    selected_cipher_suite: cipher_suite,
                    selected_compression_method: selected_compression_method,
                    extensions: vec![],
                });
                Ok(server_hello)
            },
            HandshakeType::Certificate => {
                let mut certificate_chain_length_bytes = [0; 4];
                reader
                    .read_exact(&mut certificate_chain_length_bytes[1..])
                    .map_err(TLSError::IO)?;
                let certificate_chain_length =
                    u32::from_be_bytes(certificate_chain_length_bytes) as usize;
                let mut certificate_chain = vec![];
                let mut bytes_read = 0;
                while bytes_read != certificate_chain_length {
                    let mut certificate_length_bytes = [0; 4];
                    reader
                        .read_exact(&mut certificate_length_bytes[1..])
                        .map_err(TLSError::IO)?;
                    let certificate_length = u32::from_be_bytes(certificate_length_bytes) as usize;

                    let mut certificate_bytes = vec![0; certificate_length];
                    reader
                        .read_exact(&mut certificate_bytes)
                        .map_err(TLSError::IO)?;

                    certificate_chain.push(X509v3Certificate::new(certificate_bytes));
                    bytes_read += certificate_length + 3;
                }

                Ok(Self::Certificate(CertificateChain::X509v3(
                    certificate_chain,
                )))
            },
            HandshakeType::ServerHelloDone => Ok(Self::ServerHelloDone),
            _ => unimplemented!("Parse {handshake_type:?} message"),
        }
    }
}

impl ClientHello {
    pub fn into_bytes(self) -> Vec<u8> {
        let mut data = vec![];
        data.push(HandshakeType::ClientHello.into());

        // Three length bytes which we will fill in later
        data.extend_from_slice(&[0, 0, 0]);

        // Client version
        data.extend_from_slice(&self.version.as_bytes());

        // 32 bytes of random data
        data.extend_from_slice(&self.client_random);

        // Session id (in case we want to resume a session)
        let session_id_length = self
            .session_id_to_resume
            .as_ref()
            .map(|id| id.len() as u8)
            .unwrap_or(0);
        data.push(session_id_length);
        if let Some(session_id) = &self.session_id_to_resume {
            data.extend_from_slice(session_id)
        }

        // List supported cipher suites
        data.extend_from_slice(&(2 * self.supported_cipher_suites.len() as u16).to_be_bytes()); // we support 1 cipher which takes up two bytes
        for suite in self.supported_cipher_suites {
            let suite_bytes: [u8; 2] = suite.into();
            data.extend_from_slice(suite_bytes.as_slice());
        }

        // Compression method
        // Since compression can compromise security (see CRIME), we will
        // never use compression.
        data.push(0x01); // 1 byte of compression info
        data.push(0x00); // no compression

        // Extensions supported by the client
        let mut extension_data = vec![];

        // Server name extension
        for extension in self.extensions {
            extension_data.extend_from_slice(&extension.into_bytes());
        }
        // extension_data.extend_from_slice(&server_name_extension(hostname));
        // extension_data.extend_from_slice(&status_request_extension());
        // extension_data.extend_from_slice(&renegotiation_info_extension());
        // extension_data.extend_from_slice(&signed_certificate_timestamp_extension());

        // TODO: extensions temporarily disabled
        let extension_length = (extension_data.len() as u16).to_be_bytes();
        data.extend_from_slice(&extension_length);
        data.extend_from_slice(&extension_data);

        // Write the final length into bytes 1-3
        let data_length = data.len() as u32 - 4;
        data[1..4].copy_from_slice(&data_length.to_be_bytes()[1..]);

        data
    }
}

impl Extension {
    pub fn into_bytes(self) -> Vec<u8> {
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
