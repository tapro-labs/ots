pub struct Time {
    seconds: usize,
}

// not safe for large values
impl Time {
    pub fn from_seconds(seconds: usize) -> Self {
        Self { seconds }
    }

    pub fn from_minutes(minutes: usize) -> Self {
        Self::from_seconds(60 * minutes)
    }

    pub fn from_hours(hours: usize) -> Self {
        Self::from_minutes(60 * hours)
    }

    pub fn from_days(days: usize) -> Self {
        Self::from_hours(24 * days)
    }
}

impl Time {
    pub fn as_seconds(&self) -> usize {
        self.seconds
    }

    pub fn as_ms(&self) -> usize {
        self.as_seconds() * 1000
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_seconds_keeps_value() {
        assert_eq!(Time::from_seconds(0).as_seconds(), 0);
        assert_eq!(Time::from_seconds(1).as_seconds(), 1);
        assert_eq!(Time::from_seconds(86_400).as_seconds(), 86_400);
    }

    #[test]
    fn test_units_convert_to_seconds() {
        assert_eq!(Time::from_minutes(1).as_seconds(), 60);
        assert_eq!(Time::from_minutes(30).as_seconds(), 1_800);
        assert_eq!(Time::from_hours(1).as_seconds(), 3_600);
        assert_eq!(Time::from_hours(24).as_seconds(), 86_400);
        assert_eq!(Time::from_days(1).as_seconds(), 86_400);
        assert_eq!(Time::from_days(5).as_seconds(), 432_000);
    }

    #[test]
    fn test_equivalent_durations_are_equal() {
        assert_eq!(
            Time::from_days(2).as_seconds(),
            Time::from_hours(48).as_seconds()
        );
        assert_eq!(
            Time::from_hours(2).as_seconds(),
            Time::from_minutes(120).as_seconds()
        );
    }

    #[test]
    fn test_zero_in_every_unit_is_zero() {
        assert_eq!(Time::from_minutes(0).as_ms(), 0);
        assert_eq!(Time::from_hours(0).as_ms(), 0);
        assert_eq!(Time::from_days(0).as_ms(), 0);
    }

    #[test]
    fn test_as_ms_multiplies_by_one_thousand() {
        assert_eq!(Time::from_seconds(1).as_ms(), 1_000);
        assert_eq!(Time::from_minutes(5).as_ms(), 300_000);
        assert_eq!(Time::from_days(1).as_ms(), 86_400_000);
    }

    #[test]
    fn test_longest_secret_expiry_fits_in_redis_pexpire_argument() {
        // The redis store converts as_ms() to i64 for PEXPIRE.
        let ms = Time::from_days(5).as_ms();

        assert!(i64::try_from(ms).is_ok());
    }
}
