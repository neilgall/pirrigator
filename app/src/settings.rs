extern crate config;
extern crate serde;

use config::{Config, ConfigError, File, FileFormat};

use crate::button::{ButtonSettings};
use crate::weather::WeatherSensorSettings;
use crate::moisture::{ADCSettings, MoistureSensorSettings};
use crate::valve::ValveSettings;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Database {
	pub path: String
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Settings {
	pub database: Database,
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
