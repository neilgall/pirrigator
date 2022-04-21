extern crate config;
extern crate serde;

pub mod controller;

mod button;
mod database;
mod moisture;
mod valve;
mod weather;

pub use button::ButtonSettings;
pub use database::DatabaseSettings;
pub use moisture::{ADCSettings, MoistureSensorSettings};
pub use valve::ValveSettings;
pub use weather::WeatherSensorSettings;

use config::{Config, ConfigError, File, FileFormat};
use controller::ControllerSettings;

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
