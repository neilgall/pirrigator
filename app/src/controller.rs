use std::sync::mpsc;
use std::time::Duration;

use crate::button::{Buttons, ButtonEvent, Transition};
use crate::database::Database;
use crate::event::Event;
use crate::moisture::MoistureSensor;
use crate::valve::Valves;
use crate::weather::WeatherSensor;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct ControllerSettings {
	pub irrigate_seconds: u64
}

pub struct Controller {
	pub settings: ControllerSettings,
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
			let duration = Duration::from_secs(self.settings.irrigate_seconds);
			self.valves.irrigate_all(duration);
		}
	}
}
