extern crate config;
extern crate serde;

use config::{Config, ConfigError, Environment, File};

use crate::weather::WeatherSensorSettings;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Database {
	pub path: String
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct ADC {
	pub device: String,
	pub device_type: String,
	pub enable_gpio: u8
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct MoistureSensor {
	pub name: String,
	pub channel: u8
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct WaterValve {
	pub name: String,
	pub gpio: u8
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Button {
	pub gpio: u8
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Settings {
	pub database: Database,
	pub weather: Option<WeatherSensorSettings>,
	pub adc: Option<ADC>,
	pub buttons: Vec<Button>,
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
