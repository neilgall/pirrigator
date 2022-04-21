use chrono::{DateTime, Utc};
use std::error::Error;
use influxdb::Client;

use crate::event::Event;
use crate::event::button::ButtonEvent;
use crate::event::irrigate::IrrigatedEvent;
use crate::event::weather::WeatherEvent;
use crate::event::moisture::{Measurement, MoistureEvent};
use crate::settings::DatabaseSettings;

pub struct Database {
	client: Client
}

pub type DbResult<T> = Result<T, Box<dyn Error>>;

impl Database {
	pub fn new(settings: &DatabaseSettings) -> DbResult<Self> {
		let client = Client::new(&settings.url, &settings.database)
			.with_auth(&settings.username, &settings.password);

		Ok(Database {
			client
		})
	}

	pub fn clone(&self) -> Database {
		Database {
			client: self.client.clone()
		}
	}

	pub fn store_event(&mut self, event: &Event) -> DbResult<()> {
		match event {
			Event::ButtonEvent(b) => self.store_button(b),
			Event::WeatherEvent(w) => self.store_weather(w),
			Event::MoistureEvent(m) => self.store_moisture(m),
			Event::IrrigatedEvent(i) => self.store_irrigated(i),
			_ => Ok(())
		}?;
		Ok(())
	}

	fn store_button(&mut self, event: &ButtonEvent) -> DbResult<()> {
		Ok(())
	}

	fn store_weather(&mut self, event: &WeatherEvent) -> DbResult<()> {
		Ok(())
	}

	fn store_moisture(&mut self, event: &MoistureEvent) -> DbResult<()> {
		Ok(())
	}

	pub fn store_irrigated(&mut self, event: &IrrigatedEvent) -> DbResult<()> {
		Ok(())
	}

	pub fn get_min_moisture_in_last_hour(&self, sensor: &str) -> DbResult<Measurement> {
		Ok(0)
	}
}
