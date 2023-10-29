use super::consts;

pub type YearRange = i32;

/// A specific day in history
///
/// # Note
/// Internally, a year begins on `March 01`. This is convenient because
/// it makes leap days (`February 29`) the last day of a leap year.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    year: YearRange,
    month: u8,
    day: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Year(YearRange);

/// A Month within a year
///
/// # Note
/// The internal representation is an index into the range `[Jan, Dec]`, **not**
/// the `[Mar, Feb]` used in [Date].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Month(u8);

/// A specific day within a week
///
/// # Note
/// The internal representation is an index into the range `[Sun, Sat]`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Weekday(u8);

impl Year {
    pub const fn new(year: YearRange) -> Self {
        Self(year)
    }

    /// <https://howardhinnant.github.io/date_algorithms.html#is_leap>
    pub const fn is_leap_year(&self) -> bool {
        self.0 % 4 == 0 && (self.0 % 100 != 0 || self.0 % 400 == 0)
    }
}

impl Month {
    pub const JANUARY: Self = Self(0);
    pub const FEBRUARY: Self = Self(1);
    pub const MARCH: Self = Self(2);
    pub const APRIL: Self = Self(3);
    pub const MAY: Self = Self(4);
    pub const JUNE: Self = Self(5);
    pub const JULY: Self = Self(6);
    pub const AUGUST: Self = Self(7);
    pub const SEPTEMBER: Self = Self(8);
    pub const OCTOBER: Self = Self(9);
    pub const NOVEMBER: Self = Self(10);
    pub const DECEMBER: Self = Self(11);

    /// Construct a month from its numeric index (0 based)
    ///
    /// # Panic
    /// This function panics if `index > 11`
    pub const fn from_index(index: u8) -> Self {
        assert!((index as usize) < consts::MONTHS_PER_YEAR);
        Self(index)
    }

    /// Return the number of days in the month
    pub const fn num_days(&self, is_leap_year: bool) -> u64 {
        if is_leap_year {
            days_in_month_leap_year(self.0)
        } else {
            days_in_month_common_year(self.0)
        }
    }

    pub const fn name(&self) -> &'static str {
        match self.0 {
            0 => "Jan",
            1 => "Feb",
            2 => "Mar",
            3 => "Apr",
            4 => "May",
            5 => "Jun",
            6 => "Jul",
            7 => "Aug",
            8 => "Sep",
            9 => "Oct",
            10 => "Nov",
            11 => "Dec",
            _ => unreachable!(),
        }
    }

    const fn from_internal(internal: u8) -> Self {
        let civil_month_index = if internal < 11 {
            internal + 2
        } else {
            internal - 10
        };

        Self::from_index(civil_month_index)
    }
}

impl Weekday {
    pub const SUNDAY: Self = Self(0);
    pub const MONDAY: Self = Self(1);
    pub const TUESDAY: Self = Self(2);
    pub const WEDNESDAY: Self = Self(3);
    pub const THURSDAY: Self = Self(4);
    pub const FRIDAY: Self = Self(5);
    pub const SATURDAY: Self = Self(6);

    pub const fn new(index: u8) -> Self {
        assert!(index < 7);
        Self(index)
    }

    pub const fn name(&self) -> &'static str {
        match self.0 {
            0 => "Sun",
            1 => "Mon",
            2 => "Tue",
            3 => "Wed",
            4 => "Thu",
            5 => "Fri",
            6 => "Sat",
            _ => unreachable!(),
        }
    }
}
impl Date {
    pub const UNIX: Self = Self::from_ymd(Year(1970), Month::JANUARY, 1);

    /// Compute the date give the number of days since `Jan 1, 1970`
    ///
    /// <https://howardhinnant.github.io/date_algorithms.html#civil_from_days>
    pub const fn new_from_days_since_unix(days: i32) -> Self {
        let days = days + 719468;

        let era = days.div_euclid(consts::DAYS_PER_400_YEARS as i32);

        // Offset into the 400 year cycle, in days
        let era_offset = days.rem_euclid(consts::DAYS_PER_400_YEARS as i32);

        // Offset into the 400 year cycle, in years
        let era_year =
            (era_offset - era_offset / 1460 + era_offset / 36524 - era_offset / 146096) / 365;

        let year = era * 400 + era_year;

        let day_of_year = era_offset - (365 * era_year + era_year / 4 - era_year / 100);

        let month = month_from_day_of_year(day_of_year);

        let day = day_of_year - (153 * month + 2) / 5 + 1;

        Self {
            year,
            month: month as u8,
            day: day as u8,
        }
    }

    /// Return number of days since [Date::UNIX].
    ///
    /// <https://howardhinnant.github.io/date_algorithms.html#days_from_civil>
    pub const fn days_since_unix(&self) -> u64 {
        let era = self.year.div_euclid(400) as usize;

        // Offset into the 400 year cycle, in years
        let era_year = self.year.rem_euclid(400) as usize;

        let day_of_year = (153 * self.month as usize + 2) / 5 + self.day as usize - 1;
        let day_of_era = era_year * 365 + era_year / 4 - era_year / 100 + day_of_year;

        let days_since_unix = era * consts::DAYS_PER_400_YEARS + day_of_era - 719468;

        days_since_unix as u64
    }

    pub const fn year(&self) -> Year {
        let civil_year = if self.month < 10 {
            self.year
        } else {
            self.year + 1
        };

        Year(civil_year)
    }

    pub const fn month(&self) -> Month {
        Month::from_internal(self.month)
    }

    pub const fn weekday(&self) -> Weekday {
        let days_since_unix = self.days_since_unix();
        let index = (days_since_unix + 4).rem_euclid(7);
        Weekday::new(index as u8)
    }

    pub const fn from_ymd(year: Year, month: Month, day: u8) -> Self {
        let (year, month) = if month.0 < 2 {
            (year.0 - 1, month.0 + 10)
        } else {
            (year.0, month.0 - 2)
        };

        Self { year, month, day }
    }
}

/// <https://howardhinnant.github.io/date_algorithms.html#Computing%20month%20from%20day-of-year>
#[inline]
const fn month_from_day_of_year(day: i32) -> i32 {
    debug_assert!(!day.is_negative());
    debug_assert!((day as usize) < consts::DAYS_PER_LEAP_YEAR);

    (5 * day + 2) / 153
}

/// <https://howardhinnant.github.io/date_algorithms.html#last_day_of_month_leap_year>
#[inline]
const fn days_in_month_leap_year(month: u8) -> u64 {
    const N_DAYS_IN_MONTH_LEAP_YEAR: [u64; consts::MONTHS_PER_YEAR] =
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    N_DAYS_IN_MONTH_LEAP_YEAR[(month - 1) as usize]
}

/// <https://howardhinnant.github.io/date_algorithms.html#last_day_of_month_common_year>
#[inline]
const fn days_in_month_common_year(month: u8) -> u64 {
    const N_DAYS_IN_MONTH_COMMON_YEAR: [u64; consts::MONTHS_PER_YEAR] =
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    N_DAYS_IN_MONTH_COMMON_YEAR[(month - 1) as usize]
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_days_since_unix() {
        assert_eq!(Date::new_from_days_since_unix(0), Date::UNIX);
    }

    #[test]
    fn days_since_unix() {
        assert_eq!(Date::UNIX.days_since_unix(), 0);
    }

    #[test]
    fn weekday() {
        assert_eq!(Date::UNIX.weekday(), Weekday::THURSDAY);
        assert_eq!(
            Date::from_ymd(Year(2023), Month::OCTOBER, 23).weekday(),
            Weekday::MONDAY
        );
    }

    #[test]
    fn month() {
        const MONTHS: [(i32, i32); consts::MONTHS_PER_YEAR] = [
            (0, 30),
            (31, 60),
            (61, 91),
            (92, 121),
            (122, 152),
            (153, 183),
            (184, 213),
            (214, 244),
            (245, 274),
            (275, 305),
            (306, 336),
            (337, 365),
        ];

        for i in 0..consts::MONTHS_PER_YEAR {
            assert_eq!(month_from_day_of_year(MONTHS[i].0) as usize, i);
            assert_eq!(month_from_day_of_year(MONTHS[i].1) as usize, i);
        }
    }
}
