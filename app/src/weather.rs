use bme280::{Bme280Device, Bme280Data};
use std::error::Error;
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::{JoinHandle, sleep};
use std::time::Duration;

use crate::event::Event;
use crate::settings::WeatherSensorSettings;

use common::time::UnixTime;
use common::weather::WeatherEvent;

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
		unix_time: UnixTime::now(),
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