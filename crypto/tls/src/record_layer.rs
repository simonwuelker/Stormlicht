//! TLS Record Layer Protocol.

use crate::connection::TLS_VERSION;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ConnectionEnd {
    Server,
    Client,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum PRFAlgorithm {
    TLS_PRF_SHA256,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BulkCipherAlgorithm {
    Null,
    RC4,
    TDES,
    AES,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CipherType {
    Stream,
    Block,
    AEAD,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum MACAlgorithm {
    Null,
    HMAC_MD5,
    HMAC_SHA1,
    HMAC_SHA256,
    HMAC_SHA384,
    HMAC_SHA512,
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// TLS Compression methods are defined in [RFC 3749](https://www.rfc-editor.org/rfc/rfc3749)
///
/// # Security
/// Encrypting compressed data can compromise security.
/// See [CRIME](https://en.wikipedia.org/wiki/CRIME) and [BREACH](https://en.wikipedia.org/wiki/BREACH)
/// for more information.
///
/// We will therefore **never** set a [CompressionMethod] other than [CompressionMethod::Null].
/// Seeing how future TLS protocol version removed this option altogether, this
/// seems like the correct approach.
pub enum CompressionMethod {
    Null,
    Deflate,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SecurityParameters {
    entity: ConnectionEnd,
    prf_algorithm: PRFAlgorithm,
    bulk_cipher_algorithm: BulkCipherAlgorithm,
    cipher_type: CipherType,
    enc_key_length: u8,
    block_length: u8,
    fixed_iv_length: u8,
    record_iv_length: u8,
    mac_algorith: MACAlgorithm,
    mac_length: u8,
    mac_key_length: u8,
    compression_algorithm: CompressionMethod,
    master_secret: [u8; 48],
    client_random: [u8; 32],
    server_random: [u8; 32],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProtocolVersion {
    pub major: u8,
    pub minor: u8,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ContentType {
    ChangeCipherSpec,
    Alert,
    Handshake,
    ApplicationData,
}

/// An uncompressed & unencrypted TLS record block.
/// The spec refers this as a `TLSPlaintext`.
#[derive(Debug)]
pub struct TLSRecord {
    content_type: ContentType,
    version: ProtocolVersion,
    length: u16,
    fragment: Vec<u8>,
}

impl TryFrom<u8> for ContentType {
    type Error = u8;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            20 => Ok(Self::ChangeCipherSpec),
            21 => Ok(Self::Alert),
            22 => Ok(Self::Handshake),
            23 => Ok(Self::ApplicationData),
            _ => Err(value),
        }
    }
}

impl From<ContentType> for u8 {
    fn from(value: ContentType) -> Self {
        match value {
            ContentType::ChangeCipherSpec => 20,
            ContentType::Alert => 21,
            ContentType::Handshake => 22,
            ContentType::ApplicationData => 23,
        }
    }
}

impl TLSRecord {
    pub fn new(content_type: ContentType, fragment: Vec<u8>) -> Self {
        assert!(fragment.len() < (1 << 15));
        Self {
            content_type: content_type,
            version: TLS_VERSION,
            length: fragment.len() as u16,
            fragment: fragment,
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![0; 5 + self.fragment.len()];
        bytes[0] = self.content_type.into();
        bytes[1..3].copy_from_slice(&self.version.as_bytes());
        bytes[3..5].copy_from_slice(&self.length.to_be_bytes());
        bytes[5..].copy_from_slice(&self.fragment);
        bytes
    }
}

impl ProtocolVersion {
    pub const fn new(major: u8, minor: u8) -> Self {
        Self { major, minor }
    }

    /// Note that the version is TLS 1.2 but the version number
    /// is slightly odd since TLS 1.0 was the successor of OpenSSL 3.0
    /// which gave it the version number [0x03, 0x01] and so on.
    pub fn as_bytes(&self) -> [u8; 2] {
        [self.major + 2, self.minor + 1]
    }
}
