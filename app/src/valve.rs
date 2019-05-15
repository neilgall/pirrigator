extern crate rustpi_io;

use rustpi_io::gpio::*;

use crate::database::Database;
use std::error::Error;
use std::time::SystemTime;

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct ValveSettings {
	pub name: String,
	pub socket: String,
	pub gpio: u8
}

enum ValveState {
	Closed,
	Open(SystemTime)
}

struct Valve {
	name: String,
	gpio: GPIO,
	state: ValveState
}

pub struct Valves {
	database: Database,
	units: Vec<Valve>,
}

impl Valve {
	fn new(s: &ValveSettings) -> Result<Self, Box<Error>> {
		let gpio = GPIO::new(s.gpio, GPIOMode::Write)?;
		Ok(Valve { 
			name: s.name.clone(),
			gpio,
			state: ValveState::Closed
		})
	}

	fn open(&mut self) -> Result<(), Box<Error>> {
		match self.state {
			ValveState::Open(_) => {
				// already open
			}
			ValveState::Closed => {
				self.gpio.set(GPIOData::High)?;
				self.state = ValveState::Open(SystemTime::now());
			}
		}
		Ok(())
	}

	fn close(&mut self, database: &Database) -> Result<(), Box<Error>> {
		match self.state {
			ValveState::Closed => {
				// already closed
			}
			ValveState::Open(opened) => {
				self.gpio.set(GPIOData::Low)?;
				self.state = ValveState::Closed;
				database.store_irrigation(&self.name, opened, SystemTime::now())?;
			}
		}
		Ok(())
	}

	fn is_open(&self) -> bool {
		match self.state {
			ValveState::Open(_) => true,
			ValveState::Closed => false
		}
	}
}

impl Valves {
	pub fn new(settings: &Vec<ValveSettings>, database: Database) -> Result<Self, Box<Error>> {
		let units: Vec<Valve> = settings.iter()
			.map(|v| Valve::new(v).unwrap())
			.collect();

		info!("Initialised {} valve(s)", units.len());

		Ok(Valves { 
			database,
			units
		})
	}

	pub fn cycle_units(&mut self) -> Result<(), Box<Error>> {
		let open = self.units.iter().position(Valve::is_open);

		// Pick the next unit to open, turning off any that is already open
		let next = match open {
			None => Some(0),
			Some(i) => {
				self.units[i].close(&self.database)?;
				if i + 1 < self.units.len() {
					Some(i + 1)
				} else {
					None
				}
			}
		};

		if let Some(i) = next {
			self.units[i].open()?;
		}

		Ok(())
	}
}