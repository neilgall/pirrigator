extern crate rustpi_io;

use rustpi_io::gpio::*;
use chrono::Utc;

use std::error::Error;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::task::{JoinHandle, spawn};
use tokio::time::sleep;

use crate::event::{Event, button::ButtonEvent};
use crate::settings::ButtonSettings;

pub struct Buttons {
	task: JoinHandle<()>
}

struct Button {
	name: String,
	gpio: GPIO
}

impl Button {
	fn new(settings: &ButtonSettings) -> Result<Self, Box<dyn Error>> {
		let gpio = GPIO::new(settings.gpio, GPIOMode::Read)?;
		Ok(Button { name: settings.name.clone(), gpio })
	}

	fn read(&self) -> bool {
		match self.gpio.value() {
			Ok(GPIOData::Low) => false,
			Ok(GPIOData::High) => true,
			Err(e) => {
				error!("button error {}", e);
				false
			}
		}
	}
}

fn read_all(buttons: &Vec<Button>) -> Vec<(&Button, bool)> {
	buttons.iter().map(|b| (b, b.read())).collect()
}

async fn main(buttons: Vec<Button>, channel: Sender<Event>) {
	info!("Started polling {} button(s)", buttons.len());

	let mut prev_values = read_all(&buttons);
	loop {
		let curr_values = read_all(&buttons);

		for change in prev_values.iter()
				.zip(curr_values.iter())
				.filter(|(prev, curr)| prev.1 != curr.1)
				.map(|(_, curr)| curr) {
			send_event(change, &channel).await;
		}

		prev_values = curr_values;
		sleep(Duration::from_millis(100)).await;
	}
}

async fn send_event(button: &(&Button, bool), channel: &Sender<Event>) {
	let event = ButtonEvent {
		time: Utc::now(),
		name: button.0.name.clone(),
		state: !button.1 // default is active high
	};

	 match channel.send(Event::ButtonEvent(event)).await {
		Ok(_) => {},
		Err(e) => error!("failed to send event {}", e)
	};
}

impl Buttons {
	pub fn new(settings: &Vec<ButtonSettings>, channel: Sender<Event>) -> Result<Self, Box<dyn Error>> {
		let buttons = settings.iter()
			.map(|b| Button::new(b).unwrap())
			.collect();
		let task = spawn(move || { main(buttons, channel) });
		Ok(Buttons { 
			task
		})
	}
}