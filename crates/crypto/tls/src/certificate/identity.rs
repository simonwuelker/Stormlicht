//! Information found in either the issuer or subject sections of an x509 certificate

use crate::{certificate, der};

use certificate::{expect_next_item, expect_type};

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

impl der::Parse for Identity {
    type Error = certificate::Error;

    fn try_from_item(item: der::Item<'_>) -> Result<Self, Self::Error> {
        let mut properties = HashSet::new();

        let sequence = expect_type!(item, Sequence)?;
        for rdn in sequence {
            let rdn = expect_type!(rdn?, Set)?;

            for key_value in rdn {
                let mut key_value = expect_type!(key_value?, Sequence)?;

                let key = expect_type!(expect_next_item!(key_value)?, ObjectIdentifier)?;
                let value = expect_next_item!(key_value)?;

                if key_value.next().is_some() {
                    return Err(certificate::Error::TrailingBytes);
                }

                // NOTE: This code might seem redundant, but technically the property value type depends on the
                //       key. In the future, we might support values that are not strings.
                let property = match key {
                    der::ObjectIdentifier::CountryName => {
                        Property::Country(parse_directory_string(value)?)
                    },
                    der::ObjectIdentifier::OrganizationName => {
                        Property::Organization(parse_directory_string(value)?)
                    },
                    der::ObjectIdentifier::OrganizationalUnitName => {
                        Property::OrganizationalUnit(parse_directory_string(value)?)
                    },
                    der::ObjectIdentifier::DistinguishedName => {
                        Property::DistinguishedName(parse_directory_string(value)?)
                    },
                    der::ObjectIdentifier::StateOrProvinceName => {
                        Property::StateOrProvince(parse_directory_string(value)?)
                    },
                    der::ObjectIdentifier::CommonName => {
                        Property::CommonName(parse_directory_string(value)?)
                    },
                    der::ObjectIdentifier::SerialNumber => {
                        Property::SerialNumber(parse_directory_string(value)?)
                    },
                    _ => {
                        // Unknown property
                        continue;
                    },
                };
                properties.insert(property);
            }
        }

        Ok(Self { properties })
    }
}

fn parse_directory_string(item: der::Item<'_>) -> Result<String, certificate::Error> {
    match item {
        der::Item::Utf8String(s) => Ok(s),
        der::Item::PrintableString(ascii_string) => Ok(ascii_string.to_string()),
        _ => Err(certificate::Error::InvalidFormat),
    }
}
