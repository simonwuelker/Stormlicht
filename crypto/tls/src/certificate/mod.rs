//! [X509 Certificate](https://www.rfc-editor.org/rfc/rfc5280) Implementation

pub mod identity;

use crate::der::{self, Parse};
pub use identity::Identity;

use sl_std::big_num::BigNum;

#[derive(Clone, Debug)]
pub struct X509Certificate {
    pub version: usize,
    pub serial_number: BigNum,
    pub signature: Signature,
    pub issuer: Identity,
}

#[derive(Clone, Debug)]
pub struct Signature {
    pub identifier: der::ObjectIdentifier,
}

#[derive(Clone, Copy, Debug)]
pub enum Error {
    InvalidFormat,
    ParsingFailed(der::Error),
    TrailingBytes,
}

macro_rules! expect_next_item {
    ($sequence: expr) => {
        $sequence
            .next()
            .map(|e| e.map_err($crate::certificate::Error::ParsingFailed))
            .ok_or($crate::certificate::Error::InvalidFormat)
            .flatten()
    };
}

macro_rules! expect_type {
    ($item: expr, $expected_type: ident) => {
        if let der::Item::$expected_type(value) = $item {
            Ok(value)
        } else {
            Err($crate::certificate::Error::InvalidFormat)
        }
    };
}

pub(crate) use {expect_next_item, expect_type};

impl der::Parse for X509Certificate {
    type Error = Error;

    fn try_from_item(item: der::Item<'_>) -> Result<Self, Self::Error> {
        // The root sequence always has the following structure:
        // * data
        // * Signature algorithm used
        // * Signature
        let mut root_sequence = match item {
            der::Item::Sequence(sequence) => sequence,
            _ => return Err(Error::InvalidFormat),
        };

        let mut certificate =
            expect_type!(root_sequence.next().ok_or(Error::InvalidFormat)??, Sequence)?;

        let version = parse_certificate_version(expect_next_item!(certificate)?)?;

        let serial_number = expect_type!(expect_next_item!(certificate)?, Integer)?.into();

        let signature = Signature::try_from_item(expect_next_item!(certificate)?)?;

        let issuer = Identity::try_from_item(expect_next_item!(certificate)?)?;

        let _signature_algorithm = expect_next_item!(root_sequence)?;

        let _signature = expect_next_item!(root_sequence)?;

        if root_sequence.next().is_some() {
            return Err(Error::InvalidFormat);
        }

        Ok(Self {
            version,
            serial_number,
            signature,
            issuer,
        })
    }
}

impl X509Certificate {
    pub fn new(bytes: &[u8]) -> Result<Self, Error> {
        let (value, remainder) = Self::try_parse(bytes)?;

        if !remainder.is_empty() {
            return Err(Error::TrailingBytes);
        }

        Ok(value)
    }
}

impl From<der::Error> for Error {
    fn from(value: der::Error) -> Self {
        Self::ParsingFailed(value)
    }
}

impl der::Parse for Signature {
    type Error = Error;

    fn try_from_item(item: der::Item<'_>) -> Result<Self, Self::Error> {
        let mut sequence = expect_type!(item, Sequence)?;
        let identifier = expect_type!(expect_next_item!(sequence)?, ObjectIdentifier)?;
        let _parameters = expect_next_item!(sequence)?;

        if sequence.next().is_some() {
            return Err(Error::TrailingBytes);
        }
        Ok(Self { identifier })
    }
}

fn parse_certificate_version(item: der::Item<'_>) -> Result<usize, Error> {
    let (version_item, _) = der::Item::parse(expect_type!(item, ContextSpecific)?)?;

    expect_type!(version_item, Integer)?
        .try_into()
        .map_err(|_| Error::InvalidFormat)
}
