//! The alert protocol as defined in the [TLS Specification](https://www.rfc-editor.org/rfc/rfc5246#section-7.2.2)

use crate::{
    encoding::{self, Cursor, Decoding, Encoding},
    enum_encoding,
};

#[derive(Clone, Copy, Debug)]
pub enum AlertError {
    UnknownAlertSeverity,
    UnknownAlertCode,
    MismatchedDataLength,
}

enum_encoding!(
    pub enum Severity(u8) {
        Warning = 1,
        Fatal = 2,
    }
);

enum_encoding!(
    pub enum Description(u8) {
        CloseNotify = 0,
        UnexpectedMessage = 10,
        BadRecordMAC = 20,
        DecryptionFailedReservedDoNotUse = 21,
        RecordOverflow = 22,
        DecompressionFailure = 30,
        HandshakeFailure = 40,
        NoCertificateReservedDoNotUse = 41,
        BadCertificate = 42,
        UnsupportedCertificate = 43,
        CertificateRevoked = 44,
        CertificateExpired = 45,
        CertificateUnknown = 46,
        IllegalParameter = 47,
        UnknownCA = 48,
        AccessDenied = 49,
        DecodeError = 50,
        DecryptError = 51,
        ExportRestrictionReservedDoNotUse = 60,
        ProtocolVersion = 70,
        InsufficientSecurity = 71,
        InternalError = 80,
        UserCanceled = 90,
        NoRenegotiation = 100,
        UnsupportedExcension = 110,
    }
);

#[derive(Clone, Copy, Debug)]
pub struct Alert {
    pub severity: Severity,
    pub description: Description,
}

impl Encoding for Alert {
    fn encode(&self, bytes: &mut Vec<u8>) {
        self.severity.encode(bytes);
        self.description.encode(bytes);
    }
}

impl<'a> Decoding<'a> for Alert {
    fn decode(cursor: &mut Cursor<'a>) -> encoding::Result<Self> {
        let severity = cursor.decode()?;
        let description = cursor.decode()?;

        let alert = Self {
            severity,
            description,
        };

        Ok(alert)
    }
}
