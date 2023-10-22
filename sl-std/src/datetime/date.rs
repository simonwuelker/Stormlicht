use super::consts;

/// A specific day in history
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    year: u64,
    day: u64,
}

impl Date {
    pub const fn new(year: u64, day: u64) -> Self {
        Self { year, day }
    }

    /// Compute the date give the number of days since `Jan 1, 1970`
    ///
    /// <https://howardhinnant.github.io/date_algorithms.html#civil_from_days>
    pub const fn new_from_n_days(days: u64) -> Self {
        let days = days + 719468;

        let era = days.div_euclid(consts::DAYS_PER_400_YEARS);

        // Offset into the 400 year cycle, in days
        let era_offset = days.rem_euclid(consts::DAYS_PER_400_YEARS);

        // Offset into the 400 year cycle, in years
        let era_year =
            (era_offset - era_offset / 1460 + era_offset / 36524 - era_offset / 146096) / 365;

        let year = era * 400 + era_year;

        let day = era_offset - (365 * era_year + era_year / 4 - era_year / 100);

        Self { year, day }
    }

    /// <https://howardhinnant.github.io/date_algorithms.html#is_leap>
    pub const fn is_leap_year(&self) -> bool {
        self.year % 4 == 0 && (self.year % 100 != 0 || self.year % 400 == 0)
    }

    /// Return the number of days in the current month
    pub const fn days_in_month(&self) -> u64 {
        let month = self.month();
        if self.is_leap_year() {
            days_in_month_leap_year(month)
        } else {
            days_in_month_common_year(month)
        }
    }

    pub const fn month(&self) -> u8 {
        month_from_day_of_year(self.day)
    }

    pub const fn from_ymd(year: u64, month: u64, day: u64) -> Self {
        Self {
            year: year,
            day: month * 30 + day,
        }
    }
}

/// <https://howardhinnant.github.io/date_algorithms.html#Computing%20month%20from%20day-of-year>
#[inline]
const fn month_from_day_of_year(day: u64) -> u8 {
    debug_assert!((day as usize) < consts::DAYS_PER_LEAP_YEAR);

    ((5 * day + 2) / 153) as u8
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
        assert_eq!(Date::new_from_n_days(0), Date::new(1970, 0));
    }

    #[test]
    fn month() {
        const MONTHS: [(u64, u64); consts::MONTHS_PER_YEAR] = [
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
