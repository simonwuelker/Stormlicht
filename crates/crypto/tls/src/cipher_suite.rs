//! TLS Cipher suites as defined in [Appendix A.5](https://www.rfc-editor.org/rfc/rfc5246#appendix-A.5)

use crate::{
    encoding::{self, Cursor, Decoding},
    enum_encoding,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum KeyExchange {
    Rsa,
    DiffieHellman,

    /// ECDH
    EllipticCurveDiffieHellman,

    /// SRP
    SecureRemotePassword,

    /// PSK
    PreSharedKey,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Authentication {
    Rsa,
    Dsa,
    Ecdsa,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum BulkEncryption {
    Rc4,
    TripleDes,
    Aes,
    Idea,
    Des,
    Camellia,
    ChaCha20,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Mac {
    Md5,
    Sha1,
    Sha256,
    Sha384,
    Sha512,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CipherSuiteParameters {
    key_exchange: Option<KeyExchange>,
    authentication: Option<Authentication>,
    bulk_encryption: Option<BulkEncryption>,
    mac: Option<Mac>,
}

impl CipherSuiteParameters {
    pub const TLS_NULL_WITH_NULL_NULL: Self = Self {
        key_exchange: None,
        authentication: None,
        bulk_encryption: None,
        mac: None,
    };

    pub const TLS_RSA_WITH_NULL_MD5: Self = Self {
        key_exchange: Some(KeyExchange::Rsa),
        authentication: Some(Authentication::Rsa),
        bulk_encryption: None,
        mac: Some(Mac::Md5),
    };

    pub const TLS_RSA_WITH_NULL_SHA: Self = Self {
        key_exchange: Some(KeyExchange::Rsa),
        authentication: Some(Authentication::Rsa),
        bulk_encryption: None,
        mac: Some(Mac::Sha1),
    };

    pub const TLS_RSA_WITH_NULL_SHA256: Self = Self {
        key_exchange: Some(KeyExchange::Rsa),
        authentication: Some(Authentication::Rsa),
        bulk_encryption: None,
        mac: Some(Mac::Sha256),
    };

    pub const TLS_RSA_WITH_RC4_128_MD5: Self = Self {
        key_exchange: Some(KeyExchange::Rsa),
        authentication: Some(Authentication::Rsa),
        bulk_encryption: Some(BulkEncryption::Rc4),
        mac: Some(Mac::Md5),
    };

    pub const TLS_RSA_WITH_RC4_128_SHA: Self = Self {
        key_exchange: Some(KeyExchange::Rsa),
        authentication: Some(Authentication::Rsa),
        bulk_encryption: Some(BulkEncryption::Rc4),
        mac: Some(Mac::Sha1),
    };

    pub const TLS_RSA_WITH_AES_128_CBC_SHA: Self = Self {
        key_exchange: Some(KeyExchange::Rsa),
        authentication: Some(Authentication::Rsa),
        bulk_encryption: Some(BulkEncryption::Aes),
        mac: Some(Mac::Sha1),
    };
}

impl<'a> Decoding<'a> for CipherSuiteParameters {
    fn decode(cursor: &mut Cursor<'a>) -> encoding::Result<Self> {
        let id: u16 = cursor.decode()?;

        let cipher_suite = match id {
            0x0000 => Self::TLS_NULL_WITH_NULL_NULL,
            0x0001 => Self::TLS_RSA_WITH_NULL_MD5,
            0x0002 => Self::TLS_RSA_WITH_NULL_SHA,
            0x003B => Self::TLS_RSA_WITH_NULL_SHA256,
            0x0004 => Self::TLS_RSA_WITH_RC4_128_MD5,
            0x0005 => Self::TLS_RSA_WITH_RC4_128_SHA,
            // ...
            0x002F => Self::TLS_RSA_WITH_AES_128_CBC_SHA,
            // ...
            _ => return Err(encoding::Error),
        };

        Ok(cipher_suite)
    }
}

enum_encoding!(
    pub enum CipherSuite(u16) {
        TlsNullWithNullNull = 0x0000,
        TlsRsaWithNullMd5 = 0x0001,
        TlsRsaWithNullSha = 0x0002,
        TlsRsaWithNullSha256 = 0x003B,
        TlsRsaWithRc4_128Md5 = 0x0004,
        TlsRsaWithRc4_128Sha = 0x0005,
        TlsRsaWith3desEdeCbcSha = 0x000A,
        TlsRsaWithAes128CbcSha = 0x002F,
        TlsRsaWithAes256CbcSha = 0x0035,
        TlsRsaWithAes128CbcSha256 = 0x003C,
        TlsRsaWithAes256CbcSha256 = 0x003D,
        TlsDhDssWith3desEdeCbcSha = 0x000D,
        TlsDhRsaWith3desEdeCbcSha = 0x0010,
        TlsDheDssWith3desEdeCbcSha = 0x0013,
        TlsDheRsaWith3desEdeCbcSha = 0x0016,
        TlsDhDssWithAes128CbcSha = 0x0030,
        TlsDhRsaWithAes128CbcSha = 0x0031,
        TlsDheDssWithAes128CbcSha = 0x0032,
        TlsDheRsaWithAes128CbcSha = 0x0033,
        TlsDhDssWithAes256CbcSha = 0x0036,
        TlsDhRsaWithAes256CbcSha = 0x0037,
        TlsDheDssWithAes256CbcSha = 0x0038,
        TlsDheRsaWithAes256CbcSha = 0x0039,
        TlsDhDssWithAes128CbcSha256 = 0x003E,
        TlsDhRsaWithAes128CbcSha256 = 0x003F,
        TlsDheDssWithAes128CbcSha256 = 0x0040,
        TlsDheRsaWithAes128CbcSha256 = 0x0067,
        TlsDhDssWithAes256CbcSha256 = 0x0068,
        TlsDhRsaWithAes256CbcSha256 = 0x0069,
        TlsDheDssWithAes256CbcSha256 = 0x006A,
        TlsDheRsaWithAes256CbcSha256 = 0x006B,
        TlsDhAnonWithRc4_128Md5 = 0x0018,
        TlsDhAnonWith3desEdeCbcSha = 0x001B,
        TlsDhAnonWithAes128CbcSha = 0x0034,
        TlsDhAnonWithAes256CbcSha = 0x003A,
        TlsDhAnonWithAes128CbcSha256 = 0x006C,
        TlsDhAnonWithAes256CbcSha256 = 0x006D,
    }
);
