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
