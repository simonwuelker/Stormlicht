//! The alert protocol as defined in the [TLS Specification](https://www.rfc-editor.org/rfc/rfc5246#section-7.2.2)

use thiserror::Error;

#[derive(Clone, Copy, Debug, Error)]
pub enum AlertError {
    #[error("Unknown alert severity: {}", .0)]
    UnknownAlertSeverity(u8),
    #[error("Unknown alert code: {}", .0)]
    UnknownAlertCode(u8),
    #[error("Mismatched data length, expected 2, found {}", .0)]
    MismatchedDataLength(usize),
}

#[derive(Clone, Copy, Debug)]
pub enum Severity {
    Warning,
    Fatal,
}

#[derive(Clone, Copy, Debug)]
pub enum Description {
    CloseNotify,
    UnexpectedMessage,
    BadRecordMAC,
    #[allow(non_camel_case_types)] // The whole point is that these stand out
    DecryptionFailed_RESERVED_DO_NOT_USE,
    RecordOverflow,
    DecompressionFailure,
    HandshakeFailure,
    #[allow(non_camel_case_types)]
    NoCertificate_RESERVED_DO_NOT_USE,
    BadCertificate,
    UnsupportedCertificate,
    CertificateRevoked,
    CertificateExpired,
    CertificateUnknown,
    IllegalParameter,
    UnknownCA,
    AccessDenied,
    DecodeError,
    DecryptError,
    #[allow(non_camel_case_types)]
    ExportRestriction_RESERVED_DO_NOT_USE,
    ProtocolVersion,
    InsufficientSecurity,
    InternalError,
    UserCanceled,
    NoRenegotiation,
    UnsupportedExcension,
}

impl TryFrom<u8> for Severity {
    type Error = AlertError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Warning),
            2 => Ok(Self::Fatal),
            _ => Err(AlertError::UnknownAlertSeverity(value)),
        }
    }
}

impl TryFrom<u8> for Description {
    type Error = AlertError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::CloseNotify),
            10 => Ok(Self::UnexpectedMessage),
            20 => Ok(Self::BadRecordMAC),
            21 => Ok(Self::DecryptionFailed_RESERVED_DO_NOT_USE),
            22 => Ok(Self::RecordOverflow),
            30 => Ok(Self::DecompressionFailure),
            40 => Ok(Self::HandshakeFailure),
            41 => Ok(Self::NoCertificate_RESERVED_DO_NOT_USE),
            42 => Ok(Self::BadCertificate),
            43 => Ok(Self::UnsupportedCertificate),
            44 => Ok(Self::CertificateRevoked),
            45 => Ok(Self::CertificateExpired),
            46 => Ok(Self::CertificateUnknown),
            47 => Ok(Self::IllegalParameter),
            48 => Ok(Self::UnknownCA),
            49 => Ok(Self::AccessDenied),
            50 => Ok(Self::DecodeError),
            51 => Ok(Self::DecryptError),
            60 => Ok(Self::ExportRestriction_RESERVED_DO_NOT_USE),
            70 => Ok(Self::ProtocolVersion),
            71 => Ok(Self::InsufficientSecurity),
            80 => Ok(Self::InternalError),
            90 => Ok(Self::UserCanceled),
            100 => Ok(Self::NoRenegotiation),
            110 => Ok(Self::UnsupportedExcension),
            _ => Err(AlertError::UnknownAlertCode(value)),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Alert {
    pub severity: Severity,
    pub description: Description,
}

impl TryFrom<&[u8]> for Alert {
    type Error = AlertError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() != 2 {
            Err(AlertError::MismatchedDataLength(value.len()))
        } else {
            Ok(Self {
                severity: Severity::try_from(value[0])?,
                description: Description::try_from(value[1])?,
            })
        }
    }
}
