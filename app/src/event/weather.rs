use std::time::SystemTime;
use crate::time::UnixTime;

pub type Temperature = f64;
pub type Humidity = f64;
pub type Pressure = f64;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WeatherEvent {
	pub unix_time: UnixTime,
	pub temperature: Temperature,
	pub humidity: Humidity,
	pub pressure: Pressure
}

impl WeatherEvent {
	pub fn timestamp(&self) -> u32 {
		self.unix_time.timestamp()
	}

	pub fn system_time(&self) -> SystemTime {
		self.unix_time.system_time()
	}
}

