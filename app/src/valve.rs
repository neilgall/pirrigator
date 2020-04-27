extern crate rustpi_io;

use rustpi_io::gpio::*;

use crate::database::Database;
use std::error::Error;
use std::sync::mpsc;
use std::thread::{JoinHandle, sleep, spawn};
use std::time::{Duration, SystemTime};

const SECONDS_BETWEEN_EVENTS: u64 = 5;

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct ValveSettings {
	pub name: String,
	pub socket: String,
	pub gpio: u8
}

enum ValveState {
	Closed,
	Open
}

enum Command {
	IrrigateAll { duration: Duration },
	Irrigate { name: String, duration: Duration }
}

struct Valve {
	name: String,
	gpio: GPIO,
	state: ValveState
}

impl Valve {
	fn new(s: &ValveSettings) -> Result<Self, Box<dyn Error>> {
		let gpio = GPIO::new(s.gpio, GPIOMode::Write)?;
		Ok(Valve { 
			name: s.name.clone(),
			gpio,
			state: ValveState::Closed
		})
	}

	fn open(&mut self) -> Result<(), Box<dyn Error>> {
		match self.state {
			ValveState::Open => {
				// already open
			}
			ValveState::Closed => {
				self.gpio.set(GPIOData::High)?;
				self.state = ValveState::Open;
			}
		}
		Ok(())
	}

	fn close(&mut self) -> Result<(), Box<dyn Error>> {
		match self.state {
			ValveState::Closed => {
				// already closed
			}
			ValveState::Open => {
				self.gpio.set(GPIOData::Low)?;
				self.state = ValveState::Closed;
			}
		}
		Ok(())
	}

	fn irrigate_event(&mut self, duration: Duration, database: &Database) -> Result<(), Box<dyn Error>> {
		self.open()?;
		let opened = SystemTime::now();
		sleep(duration);
		self.close()?;
		database.store_irrigation(&self.name, opened, SystemTime::now())?;
		sleep(Duration::from_secs(SECONDS_BETWEEN_EVENTS));
		Ok(())
	}
}

impl Drop for Valve {
	fn drop(&mut self) {
		self.close().unwrap();
	}
}

fn main(rx: mpsc::Receiver<Command>, mut valves: Vec<Valve>, database: Database) {
	loop {
		let command = rx.recv().unwrap();
		match command {
			Command::IrrigateAll { duration } => {
				for valve in &mut valves {
					valve.irrigate_event(duration, &database).unwrap();
				}
			},
			Command::Irrigate { name, duration } => {
				match valves.iter_mut().find(|ref v| v.name == name) {
					Some(valve) => valve.irrigate_event(duration, &database).unwrap(),
					None => warn!("no such valve {}", name)
				}
			}
		}
	}
}

pub struct Valves {
	thread: Option<JoinHandle<()>>,
	tx: mpsc::Sender<Command>
}

impl Drop for Valves {
	fn drop(&mut self) {
		if let Some(thread) = self.thread.take() {
			thread.join().unwrap();
		}
	}
}

impl Valves {
	pub fn new(settings: &Vec<ValveSettings>, database: Database) -> Result<Self, Box<dyn Error>> {
		let valves: Vec<Valve> = settings.iter()
			.map(|v| Valve::new(v).unwrap())
			.collect();

		let (tx, rx) = mpsc::channel();

		info!("Initialised {} valve(s)", valves.len());

		let thread = spawn(move || main(rx, valves, database));

		Ok(Valves { 
			thread: Some(thread),
			tx
		})
	}

	pub fn irrigate_all(&self, duration: Duration) {
		self.tx.send(Command::IrrigateAll { duration }).unwrap();
	}

	pub fn irrigate(&self, name: &str, duration: Duration) {
		self.tx.send(Command::Irrigate { name: name.to_string(), duration }).unwrap();
	}
}