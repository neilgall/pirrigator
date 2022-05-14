use chrono::{DateTime, Utc};

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

impl super::ToInfluxDB for WeatherEvent {
	fn to_line(&self) -> String {
		format!("weather temperature={},humidity={},pressure={} {}",
			self.temperature,
			self.humidity,
			self.pressure,
			self.time.timestamp()
		)
	}
}
