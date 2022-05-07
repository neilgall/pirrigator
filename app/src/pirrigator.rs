use std::error::Error;
use tokio::sync::mpsc;

use crate::button::Buttons;
use crate::controller::{Controller, Scheduler};
use crate::database::Database;
use crate::event::Event;
use crate::moisture::MoistureSensor;
use crate::settings::Settings;
use crate::valve::Valves;
use crate::weather::WeatherSensor;


fn traverse<T, U, E>(t: &Option<T>, f: &dyn Fn(&T) -> Result<U, E>) -> Result<Option<U>, E> {
   match t {
       None => Ok(None),
       Some(t) => f(t).map(Some)
   }
}

pub struct Pirrigator {
	controller: Controller,
	rx: mpsc::Receiver<Event>
}

impl Pirrigator {
	pub fn new(s: Settings) -> Result<Pirrigator, Box<dyn Error>> {
		let (tx, rx) = mpsc::channel(10);
		let db = Database::new(&s.database)?;

		let weather = traverse(
			&s.weather,
			&|w| WeatherSensor::new(&w, tx.clone())
		)?;

		let moisture = traverse(
			&s.adc,
			&|adc| MoistureSensor::new(&adc, &s.moisture, tx.clone())
		)?;

		let buttons = Buttons::new(&s.buttons, tx.clone())?;
		
		let valves = Valves::new(&s.valves, tx.clone())?;

		let scheduler = Scheduler::new(
			&s.controller.location,
			&s.controller.zones,
			tx.clone()
		)?;

		let mut controller = Controller {
			settings: s.controller.clone(),
			scheduler,
			database: db.clone(),
			weather,
			moisture,
			buttons,
			valves
		};

		return Ok(Pirrigator {
			controller,
			rx
		})
	}

	pub fn run(&mut self) {
		self.controller.run(&mut self.rx);
	}
}
