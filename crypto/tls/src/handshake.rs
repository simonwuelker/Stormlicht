use anyhow::Result;
use std::io::Write;

use crate::{
    connection::{ProtocolVersion, TLS_VERSION},
    random,
    record_layer::{ContentType, TLSRecord},
    CipherSuite,
};

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
    type Error = u8;

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
            _ => Err(value),
        }
    }
}

#[derive(Debug)]
pub struct Handshake {
    msg_type: HandshakeType,
    data: Vec<u8>,
}

impl Handshake {
    pub fn new(msg_type: HandshakeType, data: Vec<u8>) -> Self {
        Self {
            msg_type: msg_type,
            data: data,
        }
    }

    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        // Write record header
        let length_bytes = (self.data.len() as u32).to_be_bytes();
        let header = [
            self.msg_type.into(),
            length_bytes[1],
            length_bytes[2],
            length_bytes[3],
        ];
        writer.write_all(&header)?;

        writer.write_all(&self.data)?;
        Ok(())
    }
}

pub fn client_hello(hostname: &str) -> TLSRecord {
    let mut data = vec![];
    data.push(HandshakeType::ClientHello.into());

    // Three length bytes which we will fill in later
    data.extend_from_slice(&[0, 0, 0]);

    // Client version
    data.extend_from_slice(&TLS_VERSION.as_bytes());

    // 32 bytes of random data
    let mut rng = random::CryptographicRand::new().unwrap();
    data.extend_from_slice(&rng.next_u128().to_ne_bytes());
    data.extend_from_slice(&rng.next_u128().to_ne_bytes());

    // Session id (in case we want to resume a session)
    // Since we don't support resuming session we pass 0x00 (no session id)
    data.push(0x00);

    // List supported cipher suites
    data.extend_from_slice(&2_u16.to_be_bytes()); // we support 1 cipher which takes up two bytes
    let suite_bytes: [u8; 2] = CipherSuite::TLS_RSA_WITH_AES_128_CBC_SHA.into();
    data.extend_from_slice(suite_bytes.as_slice());

    // Compression method
    // Since compression can compromise security (see CRIME), we will
    // never use compression.
    data.push(0x01); // 1 byte of compression info
    data.push(0x00); // no compression

    // Extensions supported by the client
    let mut extension_data = vec![];

    // Server name extension
    extension_data.extend_from_slice(&server_name_extension(hostname));
    extension_data.extend_from_slice(&status_request_extension());
    extension_data.extend_from_slice(&renegotiation_info_extension());
    extension_data.extend_from_slice(&signed_certificate_timestamp_extension());

    // TODO: extensions temporarily disabled
    let extension_length = (extension_data.len() as u16).to_be_bytes();
    data.extend_from_slice(&extension_length);
    data.extend_from_slice(&extension_data);

    // Write the final length into bytes 1-3
    let data_length = data.len() as u32 - 4;
    data[1..4].copy_from_slice(&data_length.to_be_bytes()[1..]);

    // NOTE: We override the version here because some TLS server apparently fail if the version isn't 1.0
    // for the initial ClientHello
    // This is also mentioned in https://www.rfc-editor.org/rfc/rfc5246#appendix-E
    TLSRecord::from_data_and_version(ContentType::Handshake, ProtocolVersion::new(1, 0), data)
}

fn server_name_extension(hostname: &str) -> Vec<u8> {
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
}

fn status_request_extension() -> Vec<u8> {
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
}

fn renegotiation_info_extension() -> Vec<u8> {
    let mut extension_data = Vec::with_capacity(5);

    // Assigned value for renegotiation info extension
    extension_data.extend_from_slice(&[0xFF, 0x01]);

    // Status request extension length
    extension_data.extend_from_slice(&1u16.to_be_bytes());

    extension_data.push(0x00); // new connection

    extension_data
}

fn signed_certificate_timestamp_extension() -> Vec<u8> {
    let mut extension_data = Vec::with_capacity(4);

    // Assigned value for renegotiation info extension
    extension_data.extend_from_slice(&[0x00, 0x12]);

    // Status request extension length
    extension_data.extend_from_slice(&0u16.to_be_bytes());

    extension_data
}
