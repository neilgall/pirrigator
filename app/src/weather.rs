use bme280::{Bme280Device, Bme280Data};
use std::error::Error;
use std::sync::mpsc::Sender;
use std::thread::{JoinHandle, spawn, sleep};
use std::time::Duration;
use crate::event::Event;

use common::time::UnixTime;
use common::weather::WeatherEvent;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct WeatherSensorSettings {
	pub device: String,
	pub address: u16,
	pub update: u64
}

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
		sleep(period);
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
		let thread = spawn(move || { main(device, channel, period) });
		Ok(WeatherSensor { 
			thread: Some(thread)
		})
	}
}