use sl_std::big_num::BigNum;

use crate::der::{self, Parse};

#[derive(Clone, Debug)]
pub struct X509v3Certificate {
    pub version: usize,
    pub serial_number: BigNum,
    pub signature: Signature,
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

macro_rules! expect_type {
    ($item: expr, $expected_type: ident) => {
        if let der::Item::$expected_type(value) = $item {
            value
        } else {
            return Err(Error::InvalidFormat);
        }
    };
}

macro_rules! expect_next_item {
    ($sequence: expr) => {
        $sequence.next().ok_or(Error::InvalidFormat)??
    };
}

impl der::Parse for X509v3Certificate {
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

        let certificate_item = expect_next_item!(root_sequence);

        let mut certificate = match certificate_item {
            der::Item::Sequence(sequence) => sequence,
            _ => return Err(Error::InvalidFormat),
        };

        let version: usize = expect_type!(
            der::Item::parse(expect_type!(
                expect_next_item!(certificate),
                ContextSpecific
            ))?
            .0,
            Integer
        )
        .try_into()
        .map_err(|_| Error::InvalidFormat)?;

        let serial_number = expect_type!(expect_next_item!(certificate), Integer).into();

        let signature = Signature::try_from_item(expect_next_item!(certificate))?;

        let _issuer_sequence = expect_type!(expect_next_item!(certificate), Sequence);

        let _signature_algorithm = expect_next_item!(root_sequence);

        let _signature = expect_next_item!(root_sequence);

        if root_sequence.next().is_some() {
            return Err(Error::InvalidFormat);
        }

        Ok(Self {
            version,
            serial_number,
            signature,
        })
    }
}

impl X509v3Certificate {
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
        let mut sequence = expect_type!(item, Sequence);
        let identifier = expect_type!(expect_next_item!(sequence), ObjectIdentifier);
        let _parameters = expect_next_item!(sequence);
        if sequence.next().is_some() {
            return Err(Error::TrailingBytes);
        }
        Ok(Self { identifier })
    }
}
