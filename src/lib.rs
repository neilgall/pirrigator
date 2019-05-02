#[macro_use]
extern crate serde_derive;

use std::error::Error;
use std::path::Path;
use std::thread::{JoinHandle, spawn};
use std::sync::mpsc;

mod button;
mod database;
mod event;
mod moisture;
mod server;
mod valve;
mod weather;
pub mod settings;

struct Controller {
	database: database::Database,
	weather: Option<weather::WeatherSensor>,
	moisture: Option<moisture::MoistureSensor>,
	buttons: button::Buttons,
	valves: valve::Valves
}

pub struct Pirrigator {
	thread: JoinHandle<()>,
	database: database::Database
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
		let (tx, rx) = mpsc::channel();

		let weather = traverse(&s.weather, &|w|
			weather::WeatherSensor::new(&w, tx.clone())
		)?;

		let moisture = traverse(&s.adc, &|adc|
			moisture::MoistureSensor::new(&adc, &s.moisture, tx.clone())
		)?;

		let buttons = button::Buttons::new(&s.buttons, tx.clone())?;
		let valves = valve::Valves::new(&s.valves)?;

		let db = database::Database::new(Path::new(&s.database.path))?;

		let mut controller = Controller {
			database: db.clone(),
			weather,
			moisture,
			buttons,
			valves
		};

		let thread = spawn(move || controller.run(rx));

		return Ok(Pirrigator { 
			thread,
			database: db
		})
	}

	pub fn run_server(&self) {
		server::run(self.database.clone());
	}
}

impl Controller {
	pub fn run(&mut self, rx: mpsc::Receiver<event::Event>) {
		loop {
			let event = rx.recv()
				.expect("receive error");

			println!("event {:?}", event);

			self.database.store_event(&event)
				.expect("database store error");

			match event {
				event::Event::ButtonEvent(b) => self.button_event(&b),
				_ => {}
			}
		}
	}

	fn button_event(&mut self, b: &button::ButtonEvent) {
		match b.transition {
			button::Transition::Released => self.valves.toggle().unwrap(),
			_ => {}
		}
	}
}
