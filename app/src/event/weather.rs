use chrono::{DateTime, Utc};
use influxdb::InfluxDbWriteable;

pub type Temperature = f64;
pub type Humidity = f64;
pub type Pressure = f64;

#[derive(Debug, Clone, InfluxDbWriteable)]
pub struct WeatherEvent {
	pub time: DateTime<Utc>,
	pub temperature: Temperature,
	pub humidity: Humidity,
	pub pressure: Pressure
}
