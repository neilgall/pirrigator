extern crate rustpi_io;

use rustpi_io::gpio::*;

use crate::database::Database;
use std::error::Error;

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct ValveSettings {
	pub name: String,
	pub gpio: u8
}

struct Valve {
	name: String,
	gpio: GPIO
}

impl Valve {
	fn new(s: &ValveSettings) -> Result<Self, Box<Error>> {
		let gpio = GPIO::new(s.gpio, GPIOMode::Write)?;
		Ok(Valve { name: s.name.clone(), gpio })
	}
}

pub struct Valves {
	database: Database,
	units: Vec<Valve>,
	active: Option<usize>
}


impl Valves {
	pub fn new(settings: &Vec<ValveSettings>, database: Database) -> Result<Self, Box<Error>> {
		let units: Vec<Valve> = settings.iter()
			.map(|v| Valve::new(v).unwrap())
			.collect();

		info!("Initialised {} valve(s)", units.len());

		Ok(Valves { 
			database,
			units,
			active: None
		})
	}

	pub fn toggle(&mut self) -> Result<(), Box<Error>> {
		let on = match self.active {
			None => 
				Some(0),
			Some(a) => {
				self.units[a].gpio.set(GPIOData::Low)?;
				if a + 1 < self.units.len() {
					Some(a + 1)
				} else {
					None
				}
			}
		};

		match on {
			None => {},
			Some(a) => { self.units[a].gpio.set(GPIOData::High)?; }
		}
		
		self.active = on;
		Ok(())
	}
}