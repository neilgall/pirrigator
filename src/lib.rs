#[macro_use]
extern crate serde_derive;

use std::error::Error;
use std::path::Path;
use std::sync::mpsc;

mod database;
mod event;
mod weather;
pub mod settings;

use event::Event;

pub struct Pirrigator {
	database: database::Database,
	event_receiver: mpsc::Receiver<event::Event>,
	weather: Option<weather::WeatherSensor>
}

impl Pirrigator {
	pub fn new(s: settings::Settings) -> Result<Pirrigator, Box<Error>> {
		let db = database::Database::new(Path::new(&s.database.path))?;
		let (tx, rx) = mpsc::channel();

		let w = s.weather.map(|w|
			weather::WeatherSensor::new(&w, mpsc::Sender::clone(&tx))
				.expect("Failed to initialise weather sensor")
		);

		Ok(Pirrigator{
			database: db,
			event_receiver: rx,
			weather: w
		})
	}

	pub fn run(&self) {
		loop {
			let result = match self.event_receiver.recv().expect("receive error") {
				Event::WeatherEvent(w) => self.database.store_weather(&w)
			};
			result.expect("receiver error");
		}
	}
}
