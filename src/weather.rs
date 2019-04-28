use bme280::{Bme280Device, Bme280Data};
use std::error::Error;
use std::fmt::Display;
use std::sync::mpsc::Sender;
use std::thread::{JoinHandle, spawn, sleep};
use std::time::{Duration, SystemTime};
use crate::event::Event;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct WeatherSensorSettings {
	pub device: String,
	pub address: u16,
	pub update: u64
}

pub struct WeatherSensor {
	thread: JoinHandle<()>
}

#[derive(Debug, Clone, Copy)]
pub struct WeatherEvent {
	pub timestamp: SystemTime,
	pub temperature: f64,
	pub humidity: f64,
	pub pressure: f64
}

fn main(mut device: Bme280Device, channel: Sender<Event>, period: Duration) {
	loop {
		match device.read() {
			Ok(data) => send_event(data, &channel),
			Err(e) => error(e)
		};
		sleep(period);
	}
}

fn send_event(data: Bme280Data, channel: &Sender<Event>) {
	let event = WeatherEvent {
		timestamp: SystemTime::now(),
		temperature: data.temperature,
		humidity: data.humidity,
		pressure: data.pressure
	};

	 match channel.send(Event::WeatherEvent(event)) {
		Ok(_) => {},
		Err(e) => println!("ERROR! WeatherSensor: {}", e)
	};
}

fn error<E: Display>(e: E) {
	println!("ERROR! WeatherSensor: {}", e);
}

impl WeatherSensor {
	pub fn new(settings: &WeatherSensorSettings, channel: Sender<Event>) -> Result<Self, Box<Error>> {
		let device = Bme280Device::new(&settings.device, settings.address)?;
		let period = Duration::from_secs(settings.update);
		let thread = spawn(move || { main(device, channel, period) });
		Ok(WeatherSensor { thread })
	}
}