use crate::der;

#[derive(Clone, Debug)]
pub struct X509v3Certificate(Vec<u8>);

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
        $sequence
            .next()
            .ok_or(Error::InvalidFormat)?
            .map_err(Error::ParsingFailed)?
    };
}

impl X509v3Certificate {
    pub fn new(bytes: Vec<u8>) -> Result<Self, Error> {
        log::info!("bytes {:?}", &bytes[..10]);
        // The root sequence always has the following structure:
        // * data
        // * Signature algorithm used
        // * Signature
        let (root_item, root_length) = der::Item::parse(&bytes).map_err(Error::ParsingFailed)?;

        if root_length != bytes.len() {
            return Err(Error::TrailingBytes);
        }

        let mut root_sequence = match root_item {
            der::Item::Sequence(sequence) => sequence,
            _ => return Err(Error::InvalidFormat),
        };

        let certificate_item = expect_next_item!(root_sequence);

        let mut certificate = match certificate_item {
            der::Item::Sequence(sequence) => sequence,
            _ => return Err(Error::InvalidFormat),
        };

        let (version_item, _) = der::Item::parse(expect_type!(
            expect_next_item!(certificate),
            ContextSpecific
        ))
        .map_err(Error::ParsingFailed)?;

        log::info!("{:?}", version_item);

        let _signature_algorithm = expect_next_item!(root_sequence);

        let _signature = expect_next_item!(root_sequence);

        if root_sequence.next().is_some() {
            return Err(Error::InvalidFormat);
        }

        Ok(Self(bytes))
    }
}
