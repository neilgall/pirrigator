use influxdb::InfluxDbWriteable;
use chrono::{DateTime, Utc};

pub type Temperature = f64;
pub type Humidity = f64;
pub type Pressure = f64;

#[derive(Debug, Clone, Copy, InfluxDbWriteable)]
pub struct WeatherEvent {
	pub time: DateTime<Utc>,
	pub temperature: Temperature,
	pub humidity: Humidity,
	pub pressure: Pressure
}
