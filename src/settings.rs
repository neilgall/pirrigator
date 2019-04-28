extern crate config;
extern crate serde;

use config::{Config, ConfigError, Environment, File};

use crate::button::{ButtonSettings};
use crate::weather::WeatherSensorSettings;
use crate::moisture::{ADCSettings, MoistureSensorSettings};

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Database {
	pub path: String
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct WaterValve {
	pub name: String,
	pub gpio: u8
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Settings {
	pub database: Database,
	pub weather: Option<WeatherSensorSettings>,
	pub adc: Option<ADCSettings>,
	pub moisture: Vec<MoistureSensorSettings>,
	pub buttons: Vec<ButtonSettings>,
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
