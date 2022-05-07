use bme280::{Bme280Device, Bme280Data};
use std::error::Error;
use std::thread;
use std::thread::{JoinHandle};
use std::time::Duration;
use chrono::Utc;
use tokio::sync::mpsc::Sender;

use crate::event::{Event, weather::WeatherEvent};
use crate::settings::WeatherSensorSettings;

pub struct WeatherSensor {
	thread: Option<JoinHandle<()>>
}

impl Drop for WeatherSensor {
	fn drop(&mut self) {
		if let Some(thread) = self.thread.take() {
			thread.join().unwrap();
		}
	}
}

fn main(mut device: Bme280Device, channel: Sender<Event>, period: Duration) {
	info!("Started weather sensor");
	loop {
		match device.read() {
			Ok(data) => send_event(data, &channel),
			Err(e) => error!("ERROR! reading WeatherSensor: {}", e)
		};
		thread::park_timeout(period);
	}
}

fn send_event(data: Bme280Data, channel: &Sender<Event>) {
	let event = WeatherEvent {
		time: Utc::now(),
		temperature: data.temperature,
		humidity: data.humidity,
		pressure: data.pressure
	};

	 match channel.send(Event::WeatherEvent(event)) {
		Ok(_) => {},
		Err(e) => error!("ERROR! sending event from WeatherSensor: {}", e)
	};
}

impl WeatherSensor {
	pub fn new(settings: &WeatherSensorSettings, channel: Sender<Event>) -> Result<Self, Box<dyn Error>> {
		let device = Bme280Device::new(&settings.device, settings.address)?;
		let period = Duration::from_secs(settings.update);
		let thread = thread::Builder::new()
			.name("weather".to_string())
			.spawn(move || { main(device, channel, period) })?;
		Ok(WeatherSensor { 
			thread: Some(thread)
		})
	}
}