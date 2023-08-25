//! Provides various Date and Time utilities

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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DateTime {
    year: u32,
    month: Month,
    day: u32,
    hour: u32,
    minute: u32,
    seconds: u32,
}

impl DateTime {
    #[must_use]
    pub fn from_ymd_hms(
        year: u32,
        month: Month,
        day: u32,
        hour: u32,
        minute: u32,
        seconds: u32,
    ) -> Self {
        Self {
            year,
            month,
            day,
            hour,
            minute,
            seconds,
        }
    }

    #[must_use]
    pub fn weekday(&self) -> Weekday {
        let yy = self.year % 100;
        let year_code = (yy + (yy / 4)) % 7;
        let month_code = match self.month {
            Month::January => 0,
            Month::February => 3,
            Month::March => 3,
            Month::April => 6,
            Month::May => 1,
            Month::June => 4,
            Month::July => 6,
            Month::August => 2,
            Month::September => 5,
            Month::October => 0,
            Month::November => 3,
            Month::December => 5,
        };
        let century_code = match self.year / 100 {
            17 => 4,
            18 => 2,
            19 => 0,
            20 => 6,
            21 => 4,
            22 => 2,
            23 => 0,
            _ => {
                // We can only compute weekdays for the centuries named
                // above (which seems reasonable enough)
                0
            },
        };

        let leap_year_code = if self.is_leap_year() && self.month <= Month::February {
            1
        } else {
            0
        };
        let day_code = (year_code + month_code + century_code + self.day - leap_year_code) % 7;

        match day_code {
            0 => Weekday::Sunday,
            1 => Weekday::Monday,
            2 => Weekday::Tuesday,
            3 => Weekday::Wednesday,
            4 => Weekday::Thursday,
            5 => Weekday::Friday,
            6 => Weekday::Saturday,
            _ => unreachable!("day code was taken modulo 7"),
        }
    }

    #[inline]
    #[must_use]
    pub fn is_leap_year(&self) -> bool {
        self.year % 4 == 0 && !(self.year % 100 == 0 && self.year % 400 != 0)
    }

    /// Parse a [DateTime] as defined in [RFC 822](https://datatracker.ietf.org/doc/html/rfc822)
    pub fn from_rfc822(mut s: &str) -> Result<Self, ParseError> {
        // The date may or may not be prefixed with "<Weekday>,"
        let weekday = if let Some((weekday, date)) = s.split_once(',') {
            s = date.trim();
            Some(Weekday::from_rfc822(weekday)?)
        } else {
            None
        };

        let mut words = s.split_ascii_whitespace();
        let day_str = words.next().ok_or(ParseError::MissingDay)?;
        let month_str = words.next().ok_or(ParseError::MissingMonth)?;
        let year_str = words.next().ok_or(ParseError::MissingYear)?;

        if day_str.len() > 2 {
            return Err(ParseError::InvalidDay);
        } else if year_str.len() != 2 {
            return Err(ParseError::InvalidYear);
        }

        let day = day_str.parse::<u32>().map_err(|_| ParseError::InvalidDay)?;
        let month = Month::from_rfc822(month_str)?;
        let year = year_str
            .parse::<u32>()
            .map_err(|_| ParseError::InvalidYear)?
            + 1900;

        let mut time_segments = words.next().ok_or(ParseError::MissingTime)?.split(':');

        let parse_time_segment = |s: &str, err| {
            if s.len() != 2 {
                return Err(err);
            }
            s.parse::<u32>().map_err(|_| err)
        };

        let hour_str = time_segments.next().ok_or(ParseError::MissingHour)?;
        let hour = parse_time_segment(hour_str, ParseError::InvalidHour)?;

        let minute_str = time_segments.next().ok_or(ParseError::MissingMinute)?;
        let minute = parse_time_segment(minute_str, ParseError::InvalidMinute)?;

        let seconds = if let Some(seconds_str) = time_segments.next() {
            parse_time_segment(seconds_str, ParseError::InvalidSecond)?
        } else {
            0
        };

        let parsed_date = Self {
            year,
            month,
            day,
            hour,
            minute,
            seconds,
        };

        if weekday.is_some_and(|provided_weekday| provided_weekday != parsed_date.weekday()) {
            return Err(ParseError::IncorrectWeekday);
        }

        Ok(parsed_date)
    }
}

#[cfg(test)]
mod tests {
    use super::{DateTime, Month, Weekday};

    #[test]
    fn detect_leap_year() {
        // Not a leap year
        assert!(!DateTime::from_ymd_hms(1993, Month::January, 0, 0, 0, 0).is_leap_year());

        // Leap year, 1992 is divisible by four
        assert!(DateTime::from_ymd_hms(1992, Month::January, 0, 0, 0, 0).is_leap_year());

        // Not a leap year, while 1900 is divisible by four, its also divisible by 100
        assert!(!DateTime::from_ymd_hms(1900, Month::January, 0, 0, 0, 0).is_leap_year());

        // Leap year, despite being divisible by 100 it is also divisible by 400
        assert!(DateTime::from_ymd_hms(2000, Month::January, 0, 0, 0, 0).is_leap_year());
    }

    #[test]
    fn compute_weekday() {
        assert_eq!(
            DateTime::from_ymd_hms(2023, Month::August, 25, 0, 0, 0).weekday(),
            Weekday::Friday
        );

        assert_eq!(
            DateTime::from_ymd_hms(1969, Month::July, 20, 0, 0, 0).weekday(),
            Weekday::Sunday
        );

        // 1968 was a leap year
        assert_eq!(
            DateTime::from_ymd_hms(1968, Month::April, 4, 0, 0, 0).weekday(),
            Weekday::Thursday
        );
    }

    #[test]
    fn parse_rfc822() {
        // Invalid - the 25th of August 2023 was not a Sunday
        assert!(DateTime::from_rfc822("Sun, 25 Aug 2023 19:01:32").is_err());

        assert_eq!(
            DateTime::from_rfc822("Thu, 04 Apr 68 00:00"),
            Ok(DateTime::from_ymd_hms(1968, Month::April, 4, 0, 0, 0))
        );

        assert_eq!(
            DateTime::from_rfc822("04 Apr 68 00:00"),
            Ok(DateTime::from_ymd_hms(1968, Month::April, 4, 0, 0, 0))
        );
    }
}
