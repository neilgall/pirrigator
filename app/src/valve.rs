use chrono::Utc;
use rustpi_io::gpio::*;

use std::error::Error;
use std::thread::{JoinHandle, sleep, spawn};
use std::time::Duration;
use tokio::sync::mpsc;

use crate::event::{Event, irrigate::IrrigatedEvent};
use crate::settings::ValveSettings;

const SECONDS_BETWEEN_EVENTS: u64 = 5;

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

	fn irrigate_event(&mut self, duration: Duration) -> Result<IrrigatedEvent, Box<dyn Error>> {
		self.open()?;
		let opened = Utc::now();
		sleep(duration);
		self.close()?;

		Ok(IrrigatedEvent {
			time: opened,
			name: self.name.clone(),
			seconds: duration.as_secs() as u32
		})
	}
}

impl Drop for Valve {
	fn drop(&mut self) {
		self.close().unwrap();
	}
}

async fn irrigate(valve: &mut Valve, duration: Duration, tx: &mpsc::Sender<Event>) {
	valve.irrigate_event(duration).into_iter().for_each(
		|event| tx.send(Event::IrrigatedEvent(event)).unwrap()
	);
	sleep(Duration::from_secs(SECONDS_BETWEEN_EVENTS))
}

async fn main(mut rx: mpsc::Receiver<Command>, event_tx: mpsc::Sender<Event>, mut valves: Vec<Valve>) {
	while let Some(command) = rx.recv().await {
		match command {
			Command::IrrigateAll { duration } => {
				for mut valve in &mut valves {
					irrigate(&mut valve, duration, &event_tx).await;
				}
			},
			Command::Irrigate { name, duration } => {
				match valves.iter_mut().find(|ref v| v.name == name) {
					Some(valve) => irrigate(valve, duration, &event_tx).await,
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
	pub fn new(settings: &Vec<ValveSettings>, event_tx: mpsc::Sender<Event>) -> Result<Self, Box<dyn Error>> {
		let valves: Vec<Valve> = settings.iter()
			.map(|v| Valve::new(v).unwrap())
			.collect();

		let (command_tx, command_rx) = mpsc::channel();

		info!("Initialised {} valve(s)", valves.len());

		let thread = spawn(move || main(command_rx, event_tx, valves));

		Ok(Valves { 
			thread: Some(thread),
			tx: command_tx
		})
	}

	pub async fn irrigate_all(&self, duration: Duration) {
		self.tx.send(Command::IrrigateAll { duration }).await.unwrap();
	}

	pub async fn irrigate(&self, name: &str, duration: Duration) {
		self.tx.send(Command::Irrigate { name: name.to_string(), duration }).await.unwrap();
	}
}