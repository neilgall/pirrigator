use std::time::SystemTime;

#[derive(Debug)]
pub enum Transition {
	Pressed,
	Released
}

impl From<bool> for Transition {
	fn from(b: bool) -> Self {
		// default is active high
		if b {
			Transition::Released
		} else {
			Transition::Pressed
		}
	}
}

#[derive(Debug)]
pub struct ButtonEvent {
	pub timestamp: SystemTime,
	pub name: String,
	pub transition: Transition
}
