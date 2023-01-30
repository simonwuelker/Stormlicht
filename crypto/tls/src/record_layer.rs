//! TLS Record Layer Protocol.

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

#[derive(Debug)]
pub struct TLSPlaintext {
    content_type: ContentType,
    version: ProtocolVersion,
    length: u16,
    fragment: Vec<u8>,
}

#[derive(Debug)]
pub struct TLSCompressed {
    content_type: ContentType,
    version: ProtocolVersion,
    length: u16,
    fragment: Vec<u8>,
}

#[derive(Debug)]
pub struct TLSCipherText<T> {
    content_type: ContentType,
    version: ProtocolVersion,
    length: u16,
    fragment: T,
}

#[derive(Debug)]
pub struct GenericStreamCipher {
    content: Vec<u8>,
    mac: Vec<u8>,
}

impl CompressionMethod {
    pub fn compress(&self, plaintext: TLSPlaintext) -> TLSCompressed {
        match self {
            Self::Null => TLSCompressed {
                content_type: plaintext.content_type,
                version: plaintext.version,
                length: plaintext.length,
                fragment: plaintext.fragment,
            },
            _ => todo!(),
        }
    }
}
