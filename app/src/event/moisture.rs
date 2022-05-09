use chrono::{DateTime, Utc};
use influxc::Value;

pub type Measurement = u16;

#[derive(Debug)]
pub struct MoistureEvent {
	pub time: DateTime<Utc>,
	pub name: String,
	pub value: Measurement
}

impl super::ToRecord for MoistureEvent {
	fn fill(&self, record: &mut influxc::Record) {
		record.measurement("moisture")
			.timestamp(self.time)
			.field("name", Value::from(self.name.clone()))
			.field("value", Value::from(self.value as i64));
	}
}