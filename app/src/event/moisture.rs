use chrono::{DateTime, Utc};

pub type Measurement = u16;

#[derive(Debug)]
pub struct MoistureEvent {
	pub time: DateTime<Utc>,
	pub name: String,
	pub value: Measurement
}

impl super::ToInfluxDB for MoistureEvent {
	fn to_line(&self) -> String {
		format!("moisture,name={} value={} {}",
			self.name,
			self.value,
			self.time.timestamp()
		)
	}
}