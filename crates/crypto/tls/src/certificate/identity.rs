//! Information found in either the issuer or subject sections of an x509 certificate

use crate::der;

use super::Error;

use std::collections::HashSet;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Property {
    Country(String),
    Organization(String),
    OrganizationalUnit(String),
    DistinguishedName(String),
    StateOrProvince(String),
    CommonName(String),
    SerialNumber(String),
}

/// The identity of a party
///
/// In spec-terms, this is a set of RelativeDistinguishedNames
#[derive(Clone, Debug)]
pub struct Identity {
    pub properties: HashSet<Property>,
}

impl<'a> der::Deserialize<'a> for Identity {
    type Error = Error;

    fn deserialize(deserializer: &mut der::Deserializer<'a>) -> Result<Self, Self::Error> {
        let sequence: der::Sequence = deserializer.parse()?;
        let mut deserializer = sequence.deserializer();

        let mut properties = HashSet::new();
        while !deserializer.is_exhausted() {
            let key_value_set: der::Set = deserializer.parse()?;
            let mut deserializer = key_value_set.deserializer();

            let key: der::ObjectIdentifier = deserializer.parse()?;

            // NOTE: This code might seem redundant, but technically the property value type depends on the
            //       key. In the future, we might support values that are not strings.
            let property = match key {
                der::ObjectIdentifier::CountryName => {
                    Property::Country(parse_directory_string(&mut deserializer)?)
                },
                der::ObjectIdentifier::OrganizationName => {
                    Property::Organization(parse_directory_string(&mut deserializer)?)
                },
                der::ObjectIdentifier::OrganizationalUnitName => {
                    Property::OrganizationalUnit(parse_directory_string(&mut deserializer)?)
                },
                der::ObjectIdentifier::DistinguishedName => {
                    Property::DistinguishedName(parse_directory_string(&mut deserializer)?)
                },
                der::ObjectIdentifier::StateOrProvinceName => {
                    Property::StateOrProvince(parse_directory_string(&mut deserializer)?)
                },
                der::ObjectIdentifier::CommonName => {
                    Property::CommonName(parse_directory_string(&mut deserializer)?)
                },
                der::ObjectIdentifier::SerialNumber => {
                    Property::SerialNumber(parse_directory_string(&mut deserializer)?)
                },
                _ => {
                    // Unknown property
                    continue;
                },
            };

            deserializer.expect_exhausted(Error::TrailingBytes)?;

            properties.insert(property);
        }

        let identity = Self { properties };

        Ok(identity)
    }
}

fn parse_directory_string(deserializer: &mut der::Deserializer) -> Result<String, Error> {
    let string = match deserializer.peek_item_tag()? {
        der::TypeTag::UTF8_STRING => {
            let utf8string: der::Utf8String = deserializer.parse()?;
            utf8string.into()
        },
        der::TypeTag::PRINTABLE_STRING => {
            let printable_string: der::PrintableString = deserializer.parse()?;
            printable_string.into()
        },
        _ => return Err(Error::InvalidFormat),
    };

    Ok(string)
}
