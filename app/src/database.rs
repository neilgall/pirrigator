use std::error::Error;
use influxdb::Client;

use crate::event::Event;
use crate::event::button::ButtonEvent;
use crate::event::weather::WeatherEvent;
use crate::event::moisture::{Measurement, MoistureEvent};
use crate::settings::DatabaseSettings;
use crate::time::*;

pub struct Database {
	client: Client
}

pub type DbResult<T> = Result<T, Box<dyn Error>>;

impl Database {
	pub fn new(settings: &DatabaseSettings) -> DbResult<Self> {
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

	pub fn store_event(&self, event: &Event) -> DbResult<()> {
		match event {
			Event::ButtonEvent(b) => self.store_button(b),
			Event::WeatherEvent(w) => self.store_weather(w),
			Event::MoistureEvent(m) => self.store_moisture(m),
			_ => Ok(())
		}?;
		Ok(())
	}

	fn store_button(&self, event: &ButtonEvent) -> DbResult<()> {
		Ok(())
	}

	fn store_weather(&self, event: &WeatherEvent) -> DbResult<()> {
		Ok(())
	}

	fn store_moisture(&self, event: &MoistureEvent) -> DbResult<()> {
		Ok(())
	}

	pub fn store_irrigation(&self, valve: &str, start: UnixTime, end: UnixTime) -> DbResult<()> {
		Ok(())
	}

	pub fn get_min_moisture_in_last_hour(&self, sensor: &str) -> DbResult<Measurement> {
		Ok(0)
	}
}
