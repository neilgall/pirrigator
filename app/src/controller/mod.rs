mod scheduler;

use futures::future;
use std::time::Duration;
use tokio::sync::mpsc;

use crate::button::Buttons;
use crate::database::Database;
use crate::event::Event;
use crate::event::button::ButtonEvent;
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
	pub async fn run(&mut self, rx: &mut mpsc::Receiver<Event>) {
		while let Some(event) = rx.recv().await {
			debug!("event {:?}", event);

			self.database.store_event(&event)
				.await
				.expect("database store error");

			match event {
				Event::ButtonEvent(b) => {
					self.button_event(&b).await
				}
				Event::ConditionalIrrigateEvent(name) => {
					self.conditionally_irrigate_zone_event(&name).await
				}
				Event::IrrigateEvent(name) => {
					self.irrigate_zone_event(&name).await
				}
				_ => {}
			}
		}
	}

	async fn button_event(&mut self, b: &ButtonEvent) {
		if !b.state {
			future::join_all(
				self.settings.zones.iter().map(|ref zone|
					self.irrigate_zone_event(&zone.name)
				)
			).await;
		}
	}

	fn zone_by_name(&self, name: &str) -> Option<&Zone> {
		self.settings.zones.iter().find(|z| z.name == name)
	}

	async fn irrigate_zone_event(&self, name: &str) {
		match self.zone_by_name(name) {
			Some(zone) => self.valves.irrigate(&zone.valve, zone.irrigate_duration()).await,
			None => warn!("unknown zone for irrigation: {}", name)
		}
	}

	async fn conditionally_irrigate_zone_event(&self, name: &str) {
		match self.zone_by_name(name) {
			Some(zone) => {
				if self.should_irrigate_zone(zone).await {
					self.irrigate_zone_event(&name);
				}
			},
			None => {
				warn!("unknown zone for conditional irrigation: {}", name)
			}
		}
	}

	async fn should_irrigate_zone(&self, zone: &Zone) -> bool {
		let moisture_levels = future::try_join_all(
			zone.sensors.iter()
				.map(|sensor| self.database.get_min_moisture_in_last_hour(sensor))
		).await;

		let any_below_threshold = moisture_levels
			.iter()
			.any(|result| result.map(|m| m < zone.threshold).unwrap_or(false));

		any_below_threshold
	}


	// 		debug!("zone {} below moisture threshold in past hour; starting irrigation", zone.name);
	// 		async || self.valves.irrigate(&zone.valve, zone.irrigate_duration()).await
	// 	} else {
	// 		debug!("zone {} above moisture threshold in past hour; skipping irrigation", zone.name);
	// 		async || {}
	// 	}
	// }
}
