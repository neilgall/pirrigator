use std::sync::mpsc;

use crate::button::{Buttons, ButtonEvent, Transition};
use crate::database::Database;
use crate::event::Event;
use crate::moisture::MoistureSensor;
use crate::valve::Valves;
use crate::weather::WeatherSensor;

pub struct Controller {
	pub database: Database,
	pub weather: Option<WeatherSensor>,
	pub moisture: Option<MoistureSensor>,
	pub buttons: Buttons,
	pub valves: Valves
}

impl Controller {
	pub fn run(&mut self, rx: mpsc::Receiver<Event>) {
		loop {
			let event = rx.recv()
				.expect("receive error");

			debug!("event {:?}", event);

			self.database.store_event(&event)
				.expect("database store error");

			if let Event::ButtonEvent(b) = event {
				self.button_event(&b);
			}
		}
	}

	fn button_event(&mut self, b: &ButtonEvent) {
		if let Transition::Released = b.transition {
			self.valves.cycle_units().unwrap();
		}
	}
}
