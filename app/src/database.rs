use influxc::{Client, Credentials, FileBacklog, InfluxError, Precision, Record};

use crate::event::{Event, ToRecord};
use crate::event::moisture::Measurement;
use crate::settings::DatabaseSettings;

pub struct Database {
	client: Client,
	record: Record
}

type DbResult<T> = Result<T, InfluxError>;

impl Database {
	pub fn new(settings: &DatabaseSettings) -> DbResult<Self> {
		let credentials = Credentials::from_basic(&settings.username, &settings.password);
		let client = Client::build(settings.url.clone(), credentials)
			.backlog(FileBacklog::new(&settings.backlog_path)?)
			.finish()
			.expect("unable to create InfluxDB client");

		let record = Record::new(&settings.organisation, &settings.bucket)
			.precision(Precision::Seconds);

		Ok(Database {
			client,
			record
		})
	}

	pub fn store_event(&mut self, event: &Event) -> DbResult<()> {
		match event {
			Event::ButtonEvent(b) => self.write_event(b),
			Event::WeatherEvent(w) => self.write_event(w),
			Event::MoistureEvent(m) => self.write_event(m),
			Event::IrrigatedEvent(i) => self.write_event(i),
			_ => ()
		};
		Ok(())
	}

	fn write_event<E: ToRecord>(&mut self, event: &E) {
		event.fill(&mut self.record);
		if let Err(e) = self.client.write(&mut self.record) {
			warn!("Failed to write to influxdb: {}", e);
		}
	}

	pub fn get_min_moisture_in_last_hour(&self, sensor: &str) -> Result<Measurement, Box<dyn std::error::Error>> {
		Ok(0)
	}
}
