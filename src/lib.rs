#[macro_use]
extern crate serde_derive;

use std::error::Error;
use std::path::Path;
use std::sync::mpsc;

mod button;
mod database;
mod event;
mod moisture;
mod weather;
pub mod settings;

pub struct Pirrigator {
	database: database::Database,
	event_receiver: mpsc::Receiver<event::Event>,
	weather: Option<weather::WeatherSensor>,
	moisture: Option<moisture::MoistureSensor>,
	buttons: button::Buttons
}

// Turns an Option<T> into a Result<Option<U>>
fn traverse<T, U, E>(t: &Option<T>, f: &Fn(&T) -> Result<U, E>) -> Result<Option<U>, E> {
	match t {
		None => Ok(None),
		Some(t) => f(t).map(Some)
	}
}

impl Pirrigator {
	pub fn new(s: settings::Settings) -> Result<Pirrigator, Box<Error>> {
		let db = database::Database::new(Path::new(&s.database.path))?;
		let (tx, rx) = mpsc::channel();

		let weather = traverse(&s.weather, &|w|
			weather::WeatherSensor::new(&w, mpsc::Sender::clone(&tx))
		)?;

		let moisture = traverse(&s.adc, &|adc|
			moisture::MoistureSensor::new(&adc, &s.moisture, mpsc::Sender::clone(&tx))
		)?;

		let buttons = button::Buttons::new(&s.buttons, mpsc::Sender::clone(&tx))?;

		Ok(Pirrigator{
			database: db,
			event_receiver: rx,
			weather,
			moisture,
			buttons
		})
	}

	pub fn run(&self) {
		loop {
			let event = self.event_receiver.recv()
				.expect("receive error");

			println!("event {:?}", event);

			self.database.store_event(&event)
				.expect("database store error");
		}
	}
}
