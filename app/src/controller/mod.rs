mod scheduler;
mod settings;

use std::sync::mpsc;
use std::time::Duration;

use crate::button::{Buttons, ButtonEvent, Transition};
use crate::database::Database;
use crate::event::Event;
use crate::moisture::MoistureSensor;
use crate::valve::Valves;
use crate::weather::WeatherSensor;

pub use scheduler::Scheduler;
pub use settings::ControllerSettings;

pub struct Controller {
	pub settings: ControllerSettings,
	pub scheduler: Scheduler,
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

			match event {
				Event::ButtonEvent(b) => self.button_event(&b),
				Event::ScheduleEvent(name) => self.scheduled_event(&name),
				_ => {}
			}
		}
	}

	fn irrigate_duration(&self) -> Duration {
		Duration::from_secs(self.settings.irrigate_seconds)
	}

	fn button_event(&mut self, b: &ButtonEvent) {
		if let Transition::Released = b.transition {
			self.valves.irrigate_all(self.irrigate_duration());
		}
	}

	fn scheduled_event(&mut self, name: &str) {
		match self.settings.zones.iter().find(|z| z.name == name) {
			Some(zone) => self.valves.irrigate(&zone.valve, self.irrigate_duration()),
			None => warn!("unknown zone for scheduled event: {}", name)
		}
	}
}
