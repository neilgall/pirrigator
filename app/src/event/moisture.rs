use std::time::SystemTime;
use crate::time::UnixTime;

pub type Measurement = u16;

#[derive(Debug)]
pub struct MoistureEvent {
	pub unix_time: UnixTime,
	pub name: String,
	pub value: Measurement
}

impl MoistureEvent {
	pub fn timestamp(&self) -> u32 {
		self.unix_time.timestamp()
	}
}

