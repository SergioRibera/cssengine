use std::time::Duration;

pub trait DurationUnitExt {
    fn minutes(self) -> Duration;
    fn seconds(self) -> Duration;
    fn millis(self) -> Duration;
}

impl DurationUnitExt for u64 {
    fn minutes(self) -> Duration {
        Duration::from_secs(self)
    }

    fn seconds(self) -> Duration {
        Duration::from_secs(self)
    }

    fn millis(self) -> Duration {
        Duration::from_millis(self)
    }
}
