//! [X509 Certificate](https://www.rfc-editor.org/rfc/rfc5280) Implementation

pub mod identity;

use crate::der::{self, Deserialize};
pub use identity::Identity;

use sl_std::{ascii, base64, big_num::BigNum, datetime::DateTime};

#[derive(Clone, Debug)]
pub struct X509Certificate {
    pub version: CertificateVersion,
    pub serial_number: BigNum,
    pub signature_algorithm: AlgorithmIdentifier,
    pub issuer: Identity,
    pub validity: Validity,
    pub subject: Identity,
    pub subject_public_key_info: SubjectPublicKeyInfo,
}

#[derive(Clone, Debug)]
pub struct SignedCertificate {
    certificate: X509Certificate,
    _signature_algorithm: AlgorithmIdentifier,
    _signature: der::BitString,
}

#[derive(Clone, Copy, Debug)]
pub enum CertificateVersion {
    V1,
    V2,
    V3,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AlgorithmIdentifier {
    pub identifier: der::ObjectIdentifier,
}

#[derive(Clone, Copy, Debug)]
pub struct Validity {
    pub not_before: DateTime,
    pub not_after: DateTime,
}

#[derive(Clone, Debug)]
pub struct SubjectPublicKeyInfo {
    pub algorithm: AlgorithmIdentifier,
    pub subject_public_key: der::BitString,
}

#[derive(Clone, Copy, Debug)]
pub enum Error {
    IllegalVersion,
    InvalidFormat,
    ParsingFailed(der::Error),
    TrailingBytes,
}

#[derive(Debug)]
pub enum PemParseError {
    Certificate(Error),
    MalformedPem,
}

impl<'a> der::Deserialize<'a> for X509Certificate {
    type Error = Error;

    fn deserialize(deserializer: &mut der::Deserializer<'a>) -> Result<Self, Self::Error> {
        let sequence: der::Sequence<'_> = deserializer.parse()?;
        let mut deserializer = sequence.deserializer();
        let version: CertificateVersion = deserializer.parse()?;
        let serial_number: der::Integer = deserializer.parse()?;
        let algorithm: AlgorithmIdentifier = deserializer.parse()?;
        let issuer: Identity = deserializer.parse()?;
        let validity: Validity = deserializer.parse()?;
        let subject: Identity = deserializer.parse()?;
        let subject_public_key_info: SubjectPublicKeyInfo = deserializer.parse()?;

        let certificate = Self {
            version,
            serial_number: serial_number.into(),
            signature_algorithm: algorithm,
            issuer,
            validity,
            subject,
            subject_public_key_info,
        };

        Ok(certificate)
    }
}

impl<'a> der::Deserialize<'a> for SignedCertificate {
    type Error = Error;

    fn deserialize(deserializer: &mut der::Deserializer<'a>) -> Result<Self, Self::Error> {
        let sequence: der::Sequence<'_> = deserializer.parse()?;
        let mut deserializer = sequence.deserializer();

        let certificate: X509Certificate = deserializer.parse()?;
        let algorithm: AlgorithmIdentifier = deserializer.parse()?;
        let _signature: der::BitString = deserializer.parse()?;

        deserializer.expect_exhausted(Error::TrailingBytes)?;

        if certificate.signature_algorithm != algorithm {
            log::error!(
                "The signature algorithm specified in the certificate {:?} does not match the algorithm used for the actual signature {:?}", 
                certificate.signature_algorithm,
                algorithm
            );
            return Err(Error::InvalidFormat);
        }

        let signed_certificate = Self {
            certificate,
            _signature_algorithm: algorithm,
            _signature,
        };

        Ok(signed_certificate)
    }
}

impl SignedCertificate {
    /// Validates the basic properties of a certificate
    ///
    /// Precisely, we check if the signature on a certificate is *correct* and if the certificate
    /// is valid for the current time. However, we do **not** verify that we trust the issuer
    /// of said certificate!
    pub fn is_valid(&self) -> bool {
        let now = DateTime::now();
        self.certificate.validity.not_before <= now && now <= self.certificate.validity.not_after
    }

    pub fn load_from_pem(data: &[u8]) -> Result<Self, PemParseError> {
        let str: &ascii::Str = data.try_into().map_err(|_| PemParseError::MalformedPem)?;
        let mut lines = str.trim().lines();

        // Throw away the first and last lines (those delimit the b64 data)
        lines.next().ok_or(PemParseError::MalformedPem)?;
        lines.next_back().ok_or(PemParseError::MalformedPem)?;

        let base64_data: ascii::String = lines.collect();
        let certificate_bytes =
            base64::b64decode(&base64_data).map_err(|_| PemParseError::MalformedPem)?;

        let certificate = Self::from_bytes(&certificate_bytes, Error::TrailingBytes)?;

        Ok(certificate)
    }
}

impl From<der::Error> for Error {
    fn from(value: der::Error) -> Self {
        Self::ParsingFailed(value)
    }
}

impl<'a> der::Deserialize<'a> for AlgorithmIdentifier {
    type Error = Error;

    fn deserialize(deserializer: &mut der::Deserializer<'a>) -> Result<Self, Self::Error> {
        let sequence: der::Sequence<'_> = deserializer.parse()?;
        let mut deserializer = sequence.deserializer();

        let identifier: der::ObjectIdentifier = deserializer.parse()?;

        // FIXME: The type of parameter depends on the algorithm used,
        //        we can't parse this yet
        let _parameters = deserializer.next_primitive_item()?;

        deserializer.expect_exhausted(Error::TrailingBytes)?;

        let algorithm_identifier = Self { identifier };

        Ok(algorithm_identifier)
    }
}

impl<'a> der::Deserialize<'a> for Validity {
    type Error = Error;

    fn deserialize(deserializer: &mut der::Deserializer<'a>) -> Result<Self, Self::Error> {
        let sequence: der::Sequence<'_> = deserializer.parse()?;
        let mut deserializer = sequence.deserializer();

        let not_before: der::UtcTime = deserializer.parse()?;
        let not_after: der::UtcTime = deserializer.parse()?;

        deserializer.expect_exhausted(Error::TrailingBytes)?;

        let validity = Self {
            not_before: not_before.into(),
            not_after: not_after.into(),
        };

        Ok(validity)
    }
}

impl<'a> der::Deserialize<'a> for CertificateVersion {
    type Error = Error;

    fn deserialize(deserializer: &mut der::Deserializer<'a>) -> Result<Self, Self::Error> {
        let version_num: der::Integer =
            deserializer.parse_with_explicit_tag(der::TypeTag::new(0))?;

        let version_num: usize = version_num.try_into().map_err(|_| Error::IllegalVersion)?;

        let version = match version_num {
            0 => Self::V1,
            1 => Self::V2,
            2 => Self::V3,
            _ => return Err(Error::IllegalVersion),
        };

        Ok(version)
    }
}

impl<'a> der::Deserialize<'a> for SubjectPublicKeyInfo {
    type Error = Error;

    fn deserialize(deserializer: &mut der::Deserializer<'a>) -> Result<Self, Self::Error> {
        let sequence: der::Sequence<'_> = deserializer.parse()?;
        let mut deserializer = sequence.deserializer();

        let algorithm: AlgorithmIdentifier = deserializer.parse()?;
        let subject_public_key: der::BitString = deserializer.parse()?;

        deserializer.expect_exhausted(Error::TrailingBytes)?;

        let subject_public_key_info = Self {
            algorithm,
            subject_public_key,
        };
        Ok(subject_public_key_info)
    }
}

impl From<SignedCertificate> for X509Certificate {
    fn from(value: SignedCertificate) -> Self {
        value.certificate
    }
}

impl From<Error> for PemParseError {
    fn from(value: Error) -> Self {
        Self::Certificate(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_PEM: &[u8] =
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/cert.pem"));

    #[test]
    fn parse_pem() {
        let _parsed_certificate = SignedCertificate::load_from_pem(EXAMPLE_PEM).unwrap();
    }
}
