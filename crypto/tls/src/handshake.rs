use anyhow::Result;
use std::io::Write;

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
