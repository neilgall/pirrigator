use chrono::{DateTime, Utc};
use influxc::{Record, Value};

pub type Temperature = f64;
pub type Humidity = f64;
pub type Pressure = f64;

#[derive(Debug)]
pub struct WeatherEvent {
	pub time: DateTime<Utc>,
	pub temperature: Temperature,
	pub humidity: Humidity,
	pub pressure: Pressure
}

impl super::ToRecord for WeatherEvent {
	fn fill(&self, record: &mut Record) {
		record.measurement("weather")
			.timestamp(self.time)
			.field("temperature", Value::from(self.temperature))
			.field("humidity", Value::from(self.humidity))
			.field("pressure", Value::from(self.pressure));
	}
}
