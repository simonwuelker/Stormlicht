/// A specific day in history
///
/// FIXME: This does not account for leap years and is rather inaccurate in general.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    year: u64,
    day: u64,
}

impl Date {
    pub const fn new_from_n_days(days: u64) -> Self {
        Self {
            year: days / 365,
            day: days % 365,
        }
    }

    pub const fn from_ymd(year: u64, month: u64, day: u64) -> Self {
        Self {
            year: year,
            day: month * 30 + day,
        }
    }
}
