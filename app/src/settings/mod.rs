extern crate config;
extern crate serde;

pub mod controller;

use config::{Config, ConfigError, File, FileFormat};
use controller::ControllerSettings;

use crate::weather::WeatherSensorSettings;
use crate::moisture::{ADCSettings, MoistureSensorSettings};
use crate::valve::ValveSettings;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct ButtonSettings {
	pub name: String,
	pub gpio: u8
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct DatabaseSettings {
	pub path: String
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Settings {
	pub database: DatabaseSettings,
	pub controller: ControllerSettings,
	pub weather: Option<WeatherSensorSettings>,
	pub adc: Option<ADCSettings>,
	pub moisture: Vec<MoistureSensorSettings>,
	pub buttons: Vec<ButtonSettings>,
	pub valves: Vec<ValveSettings>
}

impl Settings {
	pub fn new() -> Result<Self, ConfigError> {
		let mut config = Config::new();
		config.merge(File::new("Settings", FileFormat::Yaml))?;
		config.try_into()
	}
}
