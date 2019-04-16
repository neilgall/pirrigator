extern crate config;
extern crate serde;

use config::{ConfigError, Config, File, Environment};

#[derive(Debug, Deserialize)]
struct WeatherSensor {
	device: String,
	address: u16
}

#[derive(Debug, Deserialize)]
struct ADC {
	device: String,
	device_type: String,
	enable_gpio: u8
}

#[derive(Debug, Deserialize)]
struct MoistureSensor {
	name: String,
	channel: u8
}

#[derive(Debug, Deserialize)]
struct WaterValve {
	name: String,
	gpio: u8
}

#[derive(Debug, Deserialize)]
pub struct Settings {
	weather: WeatherSensor,
	adc: ADC,
	moisture: Vec<MoistureSensor>,
	valves: Vec<WaterValve>
}

impl Settings {
	pub fn new() -> Result<Self, ConfigError> {
		let mut s = Config::new();

		s.merge(File::with_name("Settings"))?;
		s.merge(Environment::with_prefix("PIRRIGATOR"))?;

		s.try_into()
	}
}
