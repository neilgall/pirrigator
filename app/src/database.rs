use reqwest::blocking::Client;
use crate::event::{Event, ToInfluxDB};
use crate::event::moisture::Measurement;
use crate::settings::DatabaseSettings;

pub struct Database {
	client: Client,
	write_url: String,
	auth_header: String
}

impl Database {
	pub fn new(settings: &DatabaseSettings) -> Self {
		let write_url = format!(
			"{}/api/v2/write?org={}&bucket={}&precision=s",
			settings.url, settings.organisation, settings.bucket
		);

		let auth_header = format!("Token {}", settings.token);

		Database {
			client: Client::new(),
			write_url,
			auth_header
		}
	}

	pub fn store_event(&self, event: &Event) {
		match event {
			Event::ButtonEvent(b) => self.write_event(b),
			Event::WeatherEvent(w) => self.write_event(w),
			Event::MoistureEvent(m) => self.write_event(m),
			Event::IrrigatedEvent(i) => self.write_event(i),
			_ => ()
		};
	}

	fn write_event<E: ToInfluxDB>(&self, event: &E) {
		let rsp = self.client.post(&self.write_url)
			.header("Authorization", &self.auth_header)
			.body(event.to_line())
			.send();

		debug!("influxdb>> {:?}", rsp);

		match rsp {
			Err(e) => warn!("Failed to write to influxdb: {}", e),
			Ok(e) if e.status().as_u16() > 299 => warn!("influxdb response code {}", e.status()),
			_ => {}
		}
	}

	pub fn get_min_moisture_in_last_hour(&self, sensor: &str) -> Result<Measurement, Box<dyn std::error::Error>> {
		Ok(0)
	}
}
