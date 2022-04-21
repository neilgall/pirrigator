use std::fmt;
use std::ops::{Add, Sub};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use serde::de::{Deserialize, Deserializer, Visitor};
use serde::ser::{Serialize, Serializer};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct UnixTime {
	timestamp: u32
}

impl UnixTime {
	pub fn from_timestamp(timestamp: u32) -> UnixTime {
		UnixTime { timestamp }
	}

	pub fn from(system_time: SystemTime) -> UnixTime {
		let timestamp = system_time.duration_since(UNIX_EPOCH).unwrap().as_secs() as u32;
		UnixTime { timestamp }
	}

	pub fn now() -> UnixTime {
		UnixTime::from(SystemTime::now())
	}

	pub fn timestamp(&self) -> u32 {
		self.timestamp
	}

	pub fn system_time(&self) -> SystemTime {
		UNIX_EPOCH + Duration::from_secs(self.timestamp as u64)
	}
}

impl Add<Duration> for UnixTime {
	type Output = UnixTime;

	fn add(self, duration: Duration) -> UnixTime {
		UnixTime::from(self.system_time() + duration)
	}
}

impl Sub<Duration> for UnixTime {
	type Output = UnixTime;

	fn sub(self, duration: Duration) -> UnixTime {
		UnixTime::from(self.system_time() - duration)
	}
}

impl Sub<UnixTime> for UnixTime {
	type Output = Duration;

	fn sub(self, rhs: UnixTime) -> Duration {
		Duration::from_secs((self.timestamp - rhs.timestamp) as u64)
	}
}

impl Serialize for UnixTime {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_u32(self.timestamp)
	}
}

impl<'de> Deserialize<'de> for UnixTime {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {

		struct UnixTimeVisitor;
		impl<'de> Visitor<'de> for UnixTimeVisitor {
			type Value = UnixTime;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        		formatter.write_str("a unix-epoch timestamp")
    		}

			fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
				Ok(UnixTime { timestamp: value as u32 })
			}
		}

		deserializer.deserialize_u32(UnixTimeVisitor)
	}
}

pub type TimeSeries<T> = Vec<(UnixTime, T)>;


pub struct TimePeriod {
	pub start: UnixTime,
	pub end: UnixTime
}

impl TimePeriod {
	pub fn start_seconds(&self) -> u32 {
		self.start.timestamp()
	}

	pub fn end_seconds(&self) -> u32 {
		self.end.timestamp()
	}

	pub fn last_hour() -> TimePeriod {
		let now = UnixTime::now();
		TimePeriod {
			start: now - Duration::from_secs(3600),
			end: now
		}
	}
}

