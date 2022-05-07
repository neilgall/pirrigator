use influxdb::{Client, InfluxDbWriteable};

use crate::event::Event;
use crate::event::button::ButtonEvent;
use crate::event::irrigate::IrrigatedEvent;
use crate::event::weather::WeatherEvent;
use crate::event::moisture::{Measurement, MoistureEvent};
use crate::settings::DatabaseSettings;

pub struct Database {
	client: Client
}

pub type DbResult<T> = Result<T, influxdb::Error>;

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

	pub async fn store_event(&mut self, event: &Event) -> DbResult<()> {
		let result = match event {
			Event::ButtonEvent(b) => self.store_button(b).await,
			Event::WeatherEvent(w) => self.store_weather(w).await,
			Event::MoistureEvent(m) => self.store_moisture(m).await,
			Event::IrrigatedEvent(i) => self.store_irrigated(i).await,
			_ => Ok("".into())
		};
		match result {
			Ok(_) => debug!("successfully stored event {:?}", event),
			Err(e) => error!("failed to store event {:?}: {:?}", event, e)
		}
		Ok(())
	}

	async fn store_button(&mut self, event: &ButtonEvent) -> DbResult<String> {
		self.client.query(event.clone().into_query("button")).await
	}

	async fn store_weather(&mut self, event: &WeatherEvent) -> DbResult<String> {
		self.client.query(event.clone().into_query("weather")).await
	}

	async fn store_moisture(&mut self, event: &MoistureEvent) -> DbResult<String> {
		self.client.query(event.clone().into_query("moisture")).await
	}

	async fn store_irrigated(&mut self, event: &IrrigatedEvent) -> DbResult<String> {
		self.client.query(event.clone().into_query("irrigated")).await
	}

	pub fn get_min_moisture_in_last_hour(&self, sensor: &str) -> Result<Measurement, Box<dyn std::error::Error>> {
		Ok(0)
	}
}
