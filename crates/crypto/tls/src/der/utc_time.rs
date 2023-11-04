use sl_std::{ascii, datetime::DateTime};

use super::{Error, Primitive, TypeTag};

#[derive(Clone, Copy, Debug)]
pub struct UtcTime {
    datetime: DateTime,
}

impl<'a> Primitive<'a> for UtcTime {
    type Error = Error;

    const TYPE_TAG: TypeTag = TypeTag::UTC_TIME;

    fn from_value_bytes(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        // https://datatracker.ietf.org/doc/html/rfc5280#section-4.1.2.5.1
        // NOTE: this is not compliant with the der spec itself - but since we *only*
        //       use it to parse x509 certificates, we should adhere to the spec above instead
        let string = ascii::Str::from_bytes(bytes).ok_or(Error::IllegalValue)?;
        if string.len() != 13 || string[12].to_char() != 'Z' {
            return Err(Error::IllegalValue);
        }

        let year = match str::parse(string[0..2].as_str()) {
            Ok(y @ ..50) => 2000 + y,
            Ok(y @ 50..) => 1900 + y,
            Err(_) => return Err(Error::IllegalValue),
        };

        let month = str::parse::<u8>(string[2..4].as_str()).map_err(|_| Error::IllegalValue)? - 1;

        let day = str::parse(string[4..6].as_str()).map_err(|_| Error::IllegalValue)?;
        let hour = str::parse(string[6..8].as_str()).map_err(|_| Error::IllegalValue)?;
        let minute = str::parse(string[8..10].as_str()).map_err(|_| Error::IllegalValue)?;
        let seconds = str::parse(string[10..12].as_str()).map_err(|_| Error::IllegalValue)?;

        let datetime = DateTime::from_ymd_hms(year, month, day, hour, minute, seconds)
            .ok_or(Error::IllegalValue)?;

        let utc_time = Self { datetime };

        Ok(utc_time)
    }
}

impl From<UtcTime> for DateTime {
    fn from(value: UtcTime) -> Self {
        value.datetime
    }
}
