use chrono::{DateTime, Utc};

pub type Measurement = u16;

#[derive(Debug)]
pub struct MoistureEvent {
	pub time: DateTime<Utc>,
	pub name: String,
	pub value: Measurement
}

