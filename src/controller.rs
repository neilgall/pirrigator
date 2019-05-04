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

			println!("event {:?}", event);

			self.database.store_event(&event)
				.expect("database store error");

			match event {
				Event::ButtonEvent(b) => self.button_event(&b),
				_ => {}
			}
		}
	}

	fn button_event(&mut self, b: &ButtonEvent) {
		match b.transition {
			Transition::Released => self.valves.toggle().unwrap(),
			_ => {}
		}
	}
}
