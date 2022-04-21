use bme280::{Bme280Device, Bme280Data};
use std::error::Error;
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::{JoinHandle, sleep};
use std::time::{Duration, SystemTime};

use crate::event::Event;
use crate::settings::WeatherSensorSettings;
use crate::time::UnixTime;

pub type Temperature = f64;
pub type Humidity = f64;
pub type Pressure = f64;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WeatherEvent {
	pub unix_time: UnixTime,
	pub temperature: Temperature,
	pub humidity: Humidity,
	pub pressure: Pressure
}

impl WeatherEvent {
	pub fn timestamp(&self) -> u32 {
		self.unix_time.timestamp()
	}

	pub fn system_time(&self) -> SystemTime {
		self.unix_time.system_time()
	}
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