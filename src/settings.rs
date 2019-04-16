extern crate config;
extern crate serde;

use config::{ConfigError, Config, File, Environment};

#[derive(Debug, Deserialize)]
pub struct Database {
	pub path: String
}

#[derive(Debug, Deserialize)]
pub struct WeatherSensor {
	pub device: String,
	pub address: u16
}

#[derive(Debug, Deserialize)]
pub struct ADC {
	pub device: String,
	pub device_type: String,
	pub enable_gpio: u8
}

#[derive(Debug, Deserialize)]
pub struct MoistureSensor {
	pub name: String,
	pub channel: u8
}

#[derive(Debug, Deserialize)]
pub struct WaterValve {
	pub name: String,
	pub gpio: u8
}

#[derive(Debug, Deserialize)]
pub struct Button {
	pub gpio: u8
}

#[derive(Debug, Deserialize)]
pub struct Settings {
	pub database: Database,
	pub weather: WeatherSensor,
	pub adc: ADC,
	pub button: Button,
	pub moisture: Vec<MoistureSensor>,
	pub valves: Vec<WaterValve>
}

impl Settings {
	pub fn new() -> Result<Self, ConfigError> {
		let mut s = Config::new();

		s.merge(File::with_name("Settings"))?;
		s.merge(Environment::with_prefix("PIRRIGATOR"))?;

		s.try_into()
	}
}
