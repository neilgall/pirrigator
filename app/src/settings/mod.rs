extern crate config;
extern crate serde;

pub mod controller;

use config::{Config, ConfigError, File, FileFormat};
use controller::ControllerSettings;
use common::moisture::Measurement;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct ButtonSettings {
	pub name: String,
	pub gpio: u8
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct ValveSettings {
	pub name: String,
	pub socket: String,
	pub gpio: u8
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct DatabaseSettings {
	pub path: String
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct WeatherSensorSettings {
	pub device: String,
	pub address: u16,
	pub update: u64
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct ADCSettings {
	pub device: String,
	pub device_type: String,
	pub chip_select_gpio: u8,
	pub enable_gpio: u8,
	pub update: u64
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct MoistureSensorSettings {
	pub name: String,
	pub socket: String,
	pub channel: u8,
	pub min_reading: Measurement,
	pub max_reading: Measurement
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
