//! Errors defined in the [TLS Specification](https://www.rfc-editor.org/rfc/rfc5246#section-7.2.2)

#[derive(Clone, Copy, Debug)]
pub enum ErrorAlert {
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
