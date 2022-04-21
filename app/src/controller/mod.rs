mod scheduler;

use std::sync::mpsc;
use std::time::Duration;

use crate::button::Buttons;
use crate::database::Database;
use crate::event::Event;
use crate::event::button::{ButtonEvent, Transition};
use crate::moisture::MoistureSensor;
use crate::settings::controller::{ControllerSettings, Zone};
use crate::valve::Valves;
use crate::weather::WeatherSensor;

pub use scheduler::Scheduler;

impl Zone {
	fn irrigate_duration(&self) -> Duration {
		Duration::from_secs(self.irrigate_seconds)
	}
}

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
				Event::ConditionalIrrigateEvent(name) => self.conditionally_irrigate_zone_event(&name),
				Event::IrrigateEvent(name) => self.irrigate_zone_event(&name),
				_ => {}
			}
		}
	}

	fn button_event(&mut self, b: &ButtonEvent) {
		if let Transition::Released = b.transition {
			self.settings.zones.iter().for_each(|ref zone| self.irrigate_if_below_threshold(zone));
		}
	}

	fn zone_by_name<'a>(&'a self, name: &str) -> Option<&'a Zone> {
		self.settings.zones.iter().find(|z| z.name == name)
	}

	fn irrigate_zone_event(&self, name: &str) {
		match self.zone_by_name(name) {
			Some(zone) => self.valves.irrigate(&zone.valve, zone.irrigate_duration()),
			None => warn!("unknown zone for irrigation: {}", name)
		}
	}

	fn conditionally_irrigate_zone_event(&self, name: &str) {
		match self.zone_by_name(name) {
			Some(zone) => self.irrigate_if_below_threshold(zone),
			None => warn!("unknown zone for conditional irrigation: {}", name)
		}
	}

	fn irrigate_if_below_threshold(&self, zone: &Zone) {
		let any_below_threshold =  zone.sensors.iter()
			.map(|sensor| self.database.get_min_moisture_in_last_hour(sensor))
			.any(|result| result.map(|m| m < zone.threshold).unwrap_or(false));
		if any_below_threshold {
			debug!("zone {} below moisture threshold in past hour; starting irrigation", zone.name);
			self.valves.irrigate(&zone.valve, zone.irrigate_duration());
		} else {
			debug!("zone {} above moisture threshold in past hour; skipping irrigation", zone.name);
		}
	}
}
