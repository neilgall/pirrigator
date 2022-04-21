
use std::error::Error;
use std::path::Path;

use crate::event::Event;
use crate::time::*;

use crate::moisture;
use crate::weather;

pub struct Database {
}

impl Database {
	pub fn new(path: &Path) -> Result<Self, Box<dyn Error>> {
		Ok(Database {})
	}

	pub fn clone(&self) -> Database {
		Database {}
	}

	pub fn store_event(&self, event: &Event) -> Result<(), Box<dyn Error>> {
		match event {
			Event::WeatherEvent(w) => self.store_weather(w),
			Event::MoistureEvent(m) => self.store_moisture(m),
			_ => Ok(())
		}?;
		Ok(())
	}

	fn store_weather(&self, event: &weather::WeatherEvent) -> Result<(), Box<dyn Error>> {
		Ok(())
	}

	fn store_moisture(&self, event: &moisture::MoistureEvent) -> Result<(), Box<dyn Error>> {
		Ok(())
	}

	pub fn store_irrigation(&self, valve: &str, start: UnixTime, end: UnixTime) -> Result<(), Box<dyn Error>> {
		Ok(())
	}

	pub fn get_min_moisture_in_last_hour(&self, sensor: &str) -> Result<moisture::Measurement, Box<dyn Error>> {
		Ok(0)
	}
}
