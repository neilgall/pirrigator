use std::error::Error;
use influxdb::Client;

use crate::event::Event;
use crate::event::weather::WeatherEvent;
use crate::event::moisture::{Measurement, MoistureEvent};
use crate::time::*;

use crate::moisture;
use crate::settings::DatabaseSettings;

pub struct Database {
	client: Client
}

impl Database {
	pub fn new(settings: &DatabaseSettings) -> Result<Self, Box<dyn Error>> {
		let client = Client::new(&settings.url, &settings.database);

		Ok(Database {
			client
		})
	}

	pub fn clone(&self) -> Database {
		Database {
			client: self.client.clone()
		}
	}

	pub fn store_event(&self, event: &Event) -> Result<(), Box<dyn Error>> {
		match event {
			Event::WeatherEvent(w) => self.store_weather(w),
			Event::MoistureEvent(m) => self.store_moisture(m),
			_ => Ok(())
		}?;
		Ok(())
	}

	fn store_weather(&self, event: &WeatherEvent) -> Result<(), Box<dyn Error>> {
		Ok(())
	}

	fn store_moisture(&self, event: &MoistureEvent) -> Result<(), Box<dyn Error>> {
		Ok(())
	}

	pub fn store_irrigation(&self, valve: &str, start: UnixTime, end: UnixTime) -> Result<(), Box<dyn Error>> {
		Ok(())
	}

	pub fn get_min_moisture_in_last_hour(&self, sensor: &str) -> Result<Measurement, Box<dyn Error>> {
		Ok(0)
	}
}
