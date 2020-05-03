use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub type UnixTime = u32;
pub type TimeSeries<T> = Vec<(UnixTime, T)>;

pub fn to_unix_time(t: &SystemTime) -> UnixTime {
	t.duration_since(UNIX_EPOCH).unwrap().as_secs() as UnixTime
}

pub fn to_system_time(s: UnixTime) -> SystemTime {
	UNIX_EPOCH + Duration::from_secs(s as u64)
}

pub struct TimePeriod {
	pub start: SystemTime,
	pub end: SystemTime
}

impl TimePeriod {
	pub fn start_seconds(&self) -> UnixTime { to_unix_time(&self.start) }
	pub fn end_seconds(&self) -> UnixTime { to_unix_time(&self.end) }

	pub fn last_hour() -> TimePeriod {
		let now = SystemTime::now();
		TimePeriod {
			start: now.checked_sub(Duration::from_secs(3600)).unwrap(),
			end: now
		}
	}
}

