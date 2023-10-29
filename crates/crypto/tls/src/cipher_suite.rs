//! TLS Cipher suites as defined in [Appendix A.5](https://www.rfc-editor.org/rfc/rfc5246#appendix-A.5)

use crate::connection::TLSError;

#[derive(Clone, Copy, Debug)]
#[allow(non_camel_case_types)]
pub enum CipherSuite {
    /// Used during the initial handshake (before any keys have been exchanged).
    /// **MUST NOT** be negotiated since it provides no security.
    TLS_NULL_WITH_NULL_NULL,
    TLS_RSA_WITH_NULL_MD5,
    TLS_RSA_WITH_NULL_SHA,
    TLS_RSA_WITH_NULL_SHA256,
    TLS_RSA_WITH_RC4_128_MD5,
    TLS_RSA_WITH_RC4_128_SHA,
    TLS_RSA_WITH_3DES_EDE_CBC_SHA,
    TLS_RSA_WITH_AES_128_CBC_SHA,
    TLS_RSA_WITH_AES_256_CBC_SHA,
    TLS_RSA_WITH_AES_128_CBC_SHA256,
    TLS_RSA_WITH_AES_256_CBC_SHA256,
    // Diffie-Hellman suites
    TLS_DH_DSS_WITH_3DES_EDE_CBC_SHA,
    TLS_DH_RSA_WITH_3DES_EDE_CBC_SHA,
    TLS_DHE_DSS_WITH_3DES_EDE_CBC_SHA,
    TLS_DHE_RSA_WITH_3DES_EDE_CBC_SHA,
    TLS_DH_DSS_WITH_AES_128_CBC_SHA,
    TLS_DH_RSA_WITH_AES_128_CBC_SHA,
    TLS_DHE_DSS_WITH_AES_128_CBC_SHA,
    TLS_DHE_RSA_WITH_AES_128_CBC_SHA,
    TLS_DH_DSS_WITH_AES_256_CBC_SHA,
    TLS_DH_RSA_WITH_AES_256_CBC_SHA,
    TLS_DHE_DSS_WITH_AES_256_CBC_SHA,
    TLS_DHE_RSA_WITH_AES_256_CBC_SHA,
    TLS_DH_DSS_WITH_AES_128_CBC_SHA256,
    TLS_DH_RSA_WITH_AES_128_CBC_SHA256,
    TLS_DHE_DSS_WITH_AES_128_CBC_SHA256,
    TLS_DHE_RSA_WITH_AES_128_CBC_SHA256,
    TLS_DH_DSS_WITH_AES_256_CBC_SHA256,
    TLS_DH_RSA_WITH_AES_256_CBC_SHA256,
    TLS_DHE_DSS_WITH_AES_256_CBC_SHA256,
    TLS_DHE_RSA_WITH_AES_256_CBC_SHA256,
    // Anonymous Diffie-Hellman suites
    // NOTE: These are vulnerable to MITM.
    // They must not be used unless explicitly request by the application layer
    TLS_DH_anon_WITH_RC4_128_MD5,
    TLS_DH_anon_WITH_3DES_EDE_CBC_SHA,
    TLS_DH_anon_WITH_AES_128_CBC_SHA,
    TLS_DH_anon_WITH_AES_256_CBC_SHA,
    TLS_DH_anon_WITH_AES_128_CBC_SHA256,
    TLS_DH_anon_WITH_AES_256_CBC_SHA256,
}

impl From<CipherSuite> for [u8; 2] {
    fn from(value: CipherSuite) -> Self {
        match value {
            CipherSuite::TLS_NULL_WITH_NULL_NULL => [0x00, 0x00],
            CipherSuite::TLS_RSA_WITH_NULL_MD5 => [0x00, 0x01],
            CipherSuite::TLS_RSA_WITH_NULL_SHA => [0x00, 0x02],
            CipherSuite::TLS_RSA_WITH_NULL_SHA256 => [0x00, 0x03B],
            CipherSuite::TLS_RSA_WITH_RC4_128_MD5 => [0x00, 0x04],
            CipherSuite::TLS_RSA_WITH_RC4_128_SHA => [0x00, 0x05],
            CipherSuite::TLS_RSA_WITH_3DES_EDE_CBC_SHA => [0x00, 0x0A],
            CipherSuite::TLS_RSA_WITH_AES_128_CBC_SHA => [0x00, 0x2F],
            CipherSuite::TLS_RSA_WITH_AES_256_CBC_SHA => [0x00, 0x35],
            CipherSuite::TLS_RSA_WITH_AES_128_CBC_SHA256 => [0x00, 0x3C],
            CipherSuite::TLS_RSA_WITH_AES_256_CBC_SHA256 => [0x00, 0x3D],
            CipherSuite::TLS_DH_DSS_WITH_3DES_EDE_CBC_SHA => [0x00, 0x0D],
            CipherSuite::TLS_DH_RSA_WITH_3DES_EDE_CBC_SHA => [0x00, 0x10],
            CipherSuite::TLS_DHE_DSS_WITH_3DES_EDE_CBC_SHA => [0x00, 0x13],
            CipherSuite::TLS_DHE_RSA_WITH_3DES_EDE_CBC_SHA => [0x00, 0x16],
            CipherSuite::TLS_DH_DSS_WITH_AES_128_CBC_SHA => [0x00, 0x30],
            CipherSuite::TLS_DH_RSA_WITH_AES_128_CBC_SHA => [0x00, 0x31],
            CipherSuite::TLS_DHE_DSS_WITH_AES_128_CBC_SHA => [0x00, 0x32],
            CipherSuite::TLS_DHE_RSA_WITH_AES_128_CBC_SHA => [0x00, 0x33],
            CipherSuite::TLS_DH_DSS_WITH_AES_256_CBC_SHA => [0x00, 0x36],
            CipherSuite::TLS_DH_RSA_WITH_AES_256_CBC_SHA => [0x00, 0x37],
            CipherSuite::TLS_DHE_DSS_WITH_AES_256_CBC_SHA => [0x00, 0x38],
            CipherSuite::TLS_DHE_RSA_WITH_AES_256_CBC_SHA => [0x00, 0x39],
            CipherSuite::TLS_DH_DSS_WITH_AES_128_CBC_SHA256 => [0x00, 0x3E],
            CipherSuite::TLS_DH_RSA_WITH_AES_128_CBC_SHA256 => [0x00, 0x3F],
            CipherSuite::TLS_DHE_DSS_WITH_AES_128_CBC_SHA256 => [0x00, 0x40],
            CipherSuite::TLS_DHE_RSA_WITH_AES_128_CBC_SHA256 => [0x00, 0x67],
            CipherSuite::TLS_DH_DSS_WITH_AES_256_CBC_SHA256 => [0x00, 0x68],
            CipherSuite::TLS_DH_RSA_WITH_AES_256_CBC_SHA256 => [0x00, 0x69],
            CipherSuite::TLS_DHE_DSS_WITH_AES_256_CBC_SHA256 => [0x00, 0x6A],
            CipherSuite::TLS_DHE_RSA_WITH_AES_256_CBC_SHA256 => [0x00, 0x6B],
            CipherSuite::TLS_DH_anon_WITH_RC4_128_MD5 => [0x00, 0x18],
            CipherSuite::TLS_DH_anon_WITH_3DES_EDE_CBC_SHA => [0x00, 0x1B],
            CipherSuite::TLS_DH_anon_WITH_AES_128_CBC_SHA => [0x00, 0x34],
            CipherSuite::TLS_DH_anon_WITH_AES_256_CBC_SHA => [0x00, 0x3A],
            CipherSuite::TLS_DH_anon_WITH_AES_128_CBC_SHA256 => [0x00, 0x6C],
            CipherSuite::TLS_DH_anon_WITH_AES_256_CBC_SHA256 => [0x00, 0x6D],
        }
    }
}

impl TryFrom<[u8; 2]> for CipherSuite {
    type Error = TLSError;

    fn try_from(value: [u8; 2]) -> Result<Self, Self::Error> {
        let cipher_suite = match value {
            [0x00, 0x00] => Self::TLS_NULL_WITH_NULL_NULL,
            [0x00, 0x01] => Self::TLS_RSA_WITH_NULL_MD5,
            [0x00, 0x02] => Self::TLS_RSA_WITH_NULL_SHA,
            [0x00, 0x3B] => Self::TLS_RSA_WITH_NULL_SHA256,
            [0x00, 0x04] => Self::TLS_RSA_WITH_RC4_128_MD5,
            [0x00, 0x05] => Self::TLS_RSA_WITH_RC4_128_SHA,
            [0x00, 0x0A] => Self::TLS_RSA_WITH_3DES_EDE_CBC_SHA,
            [0x00, 0x2F] => Self::TLS_RSA_WITH_AES_128_CBC_SHA,
            [0x00, 0x35] => Self::TLS_RSA_WITH_AES_256_CBC_SHA,
            [0x00, 0x3C] => Self::TLS_RSA_WITH_AES_128_CBC_SHA256,
            [0x00, 0x3D] => Self::TLS_RSA_WITH_AES_256_CBC_SHA256,
            [0x00, 0x0D] => Self::TLS_DH_DSS_WITH_3DES_EDE_CBC_SHA,
            [0x00, 0x10] => Self::TLS_DH_RSA_WITH_3DES_EDE_CBC_SHA,
            [0x00, 0x13] => Self::TLS_DHE_DSS_WITH_3DES_EDE_CBC_SHA,
            [0x00, 0x16] => Self::TLS_DHE_RSA_WITH_3DES_EDE_CBC_SHA,
            [0x00, 0x30] => Self::TLS_DH_DSS_WITH_AES_128_CBC_SHA,
            [0x00, 0x31] => Self::TLS_DH_RSA_WITH_AES_128_CBC_SHA,
            [0x00, 0x32] => Self::TLS_DHE_DSS_WITH_AES_128_CBC_SHA,
            [0x00, 0x33] => Self::TLS_DHE_RSA_WITH_AES_128_CBC_SHA,
            [0x00, 0x36] => Self::TLS_DH_DSS_WITH_AES_256_CBC_SHA,
            [0x00, 0x37] => Self::TLS_DH_RSA_WITH_AES_256_CBC_SHA,
            [0x00, 0x38] => Self::TLS_DHE_DSS_WITH_AES_256_CBC_SHA,
            [0x00, 0x39] => Self::TLS_DHE_RSA_WITH_AES_256_CBC_SHA,
            [0x00, 0x3E] => Self::TLS_DH_DSS_WITH_AES_128_CBC_SHA256,
            [0x00, 0x3F] => Self::TLS_DH_RSA_WITH_AES_128_CBC_SHA256,
            [0x00, 0x40] => Self::TLS_DHE_DSS_WITH_AES_128_CBC_SHA256,
            [0x00, 0x67] => Self::TLS_DHE_RSA_WITH_AES_128_CBC_SHA256,
            [0x00, 0x68] => Self::TLS_DH_DSS_WITH_AES_256_CBC_SHA256,
            [0x00, 0x69] => Self::TLS_DH_RSA_WITH_AES_256_CBC_SHA256,
            [0x00, 0x6A] => Self::TLS_DHE_DSS_WITH_AES_256_CBC_SHA256,
            [0x00, 0x6B] => Self::TLS_DHE_RSA_WITH_AES_256_CBC_SHA256,
            [0x00, 0x18] => Self::TLS_DH_anon_WITH_RC4_128_MD5,
            [0x00, 0x1B] => Self::TLS_DH_anon_WITH_3DES_EDE_CBC_SHA,
            [0x00, 0x34] => Self::TLS_DH_anon_WITH_AES_128_CBC_SHA,
            [0x00, 0x3A] => Self::TLS_DH_anon_WITH_AES_256_CBC_SHA,
            [0x00, 0x6C] => Self::TLS_DH_anon_WITH_AES_128_CBC_SHA256,
            [0x00, 0x6D] => Self::TLS_DH_anon_WITH_AES_256_CBC_SHA256,
            _ => {
                log::warn!("Unknown TLS cipher suite: {value:?}");
                return Err(TLSError::UnknownCipherSuite);
            },
        };
        Ok(cipher_suite)
    }
}
