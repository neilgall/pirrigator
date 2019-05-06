use std::error::Error;
use std::path::Path;
use std::sync::mpsc;
use std::thread::{JoinHandle, spawn};

use crate::button::Buttons;
use crate::controller::Controller;
use crate::database::Database;
use crate::moisture::MoistureSensor;
use crate::server;
use crate::settings::Settings;
use crate::valve::Valves;
use crate::weather::WeatherSensor;


fn traverse<T, U, E>(t: &Option<T>, f: &Fn(&T) -> Result<U, E>) -> Result<Option<U>, E> {
   match t {
       None => Ok(None),
       Some(t) => f(t).map(Some)
   }
}

pub struct Pirrigator {
	thread: JoinHandle<()>,
	database: Database
}

impl Pirrigator {
	pub fn new(s: Settings) -> Result<Pirrigator, Box<Error>> {
		let (tx, rx) = mpsc::channel();

		let weather = traverse(&s.weather, &|w| WeatherSensor::new(&w, tx.clone()))?;

		let moisture = traverse(&s.adc, &|adc| MoistureSensor::new(&adc, &s.moisture, tx.clone()))?;

		let buttons = Buttons::new(&s.buttons, tx.clone())?;
		let valves = Valves::new(&s.valves)?;

		let db = Database::new(Path::new(&s.database.path))?;

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
