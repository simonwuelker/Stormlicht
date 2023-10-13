//! Provides various Date and Time utilities

use std::time::{SystemTime, UNIX_EPOCH};

pub mod consts;
mod date;
mod time;

pub use date::Date;
pub use time::Time;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Month {
    January,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseError {
    InvalidWeekday,
    InvalidMonth,
    InvalidDay,
    InvalidYear,
    InvalidHour,
    InvalidMinute,
    InvalidSecond,
    MissingDay,
    MissingMonth,
    MissingYear,
    MissingTime,
    MissingHour,
    MissingMinute,
    IncorrectWeekday,
}

impl Weekday {
    /// Parse a [Weekday] as defined in [RFC 822](https://datatracker.ietf.org/doc/html/rfc822)
    pub fn from_rfc822(s: &str) -> Result<Self, ParseError> {
        match s {
            "Mon" => Ok(Self::Monday),
            "Tue" => Ok(Self::Tuesday),
            "Wed" => Ok(Self::Wednesday),
            "Thu" => Ok(Self::Thursday),
            "Fri" => Ok(Self::Friday),
            "Sat" => Ok(Self::Saturday),
            "Sun" => Ok(Self::Sunday),
            _ => Err(ParseError::InvalidWeekday),
        }
    }
}

impl Month {
    /// Parse a [Month] as defined in [RFC 822](https://datatracker.ietf.org/doc/html/rfc822)
    pub fn from_rfc822(m: &str) -> Result<Self, ParseError> {
        match m {
            "Jan" => Ok(Self::January),
            "Feb" => Ok(Self::February),
            "Mar" => Ok(Self::March),
            "Apr" => Ok(Self::April),
            "May" => Ok(Self::May),
            "Jun" => Ok(Self::June),
            "Jul" => Ok(Self::July),
            "Aug" => Ok(Self::August),
            "Sep" => Ok(Self::September),
            "Oct" => Ok(Self::October),
            "Nov" => Ok(Self::November),
            "Dec" => Ok(Self::December),
            _ => Err(ParseError::InvalidMonth),
        }
    }

    pub fn from_index(index: u8) -> Option<Self> {
        let month = match index {
            1 => Self::January,
            2 => Self::February,
            3 => Self::March,
            4 => Self::April,
            5 => Self::May,
            6 => Self::June,
            7 => Self::July,
            8 => Self::August,
            9 => Self::September,
            10 => Self::October,
            11 => Self::November,
            12 => Self::December,
            _ => return None,
        };

        Some(month)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DateTime {
    date: Date,
    time: Time,
}

impl DateTime {
    /// Return the current [DateTime]
    ///
    /// # Panics
    /// This function panics if the current system time is before [UNIX_EPOCH].
    #[must_use]
    pub fn now() -> Self {
        let since_unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time before unix epoch");
        Self::from_unix_timestamp(since_unix.as_secs())
    }

    #[must_use]
    pub const fn from_unix_timestamp(seconds: u64) -> Self {
        let days = seconds / consts::SECONDS_PER_DAY;
        let seconds = seconds % consts::SECONDS_PER_DAY;

        let date = Date::new_from_n_days(days + consts::DAYS_BEFORE_UNIX_TIME);
        let time = Time::new_from_n_seconds_since_midnight(seconds);

        Self { date, time }
    }

    pub fn from_ymd_hms(
        year: u64,
        month: u64,
        day: u64,
        hour: u64,
        minute: u64,
        second: u64,
    ) -> Option<Self> {
        let date = Date::from_ymd(year, month, day);
        let time = Time::from_hms(hour, minute, second)?;

        Some(Self { date, time })
    }
}
