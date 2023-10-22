pub const DAYS_BEFORE_UNIX_TIME: u64 = 719_163;

/// The civil calendar repeats itself every 400 years, so this number is suprisingly useful
pub const DAYS_PER_400_YEARS: u64 = 146097;

pub const SECONDS_PER_MINUTE: u64 = 60;
pub const SECONDS_PER_HOUR: u64 = 60 * SECONDS_PER_MINUTE;
pub const SECONDS_PER_DAY: u64 = 24 * SECONDS_PER_HOUR;

pub const MONTHS_PER_YEAR: usize = 12;
pub const DAYS_PER_LEAP_YEAR: usize = 366;
