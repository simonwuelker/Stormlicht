use super::consts;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Time {
    hour: u64,
    minute: u64,
    second: u64,
}

impl Time {
    pub const fn new_from_n_seconds_since_midnight(seconds: u64) -> Self {
        let hour = seconds / consts::SECONDS_PER_HOUR;
        let seconds = seconds % consts::SECONDS_PER_HOUR;
        let minute = seconds / consts::SECONDS_PER_MINUTE;
        let second = seconds % consts::SECONDS_PER_MINUTE;

        Self {
            hour,
            minute,
            second,
        }
    }

    pub const fn from_hms(hour: u64, minute: u64, second: u64) -> Option<Self> {
        if 23 < hour || 59 < minute || 59 < second {
            return None;
        }
        Some(Self {
            hour,
            minute,
            second,
        })
    }
}
