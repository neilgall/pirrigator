use chrono::{DateTime, Utc};
use influxc::Value;

#[derive(Debug)]
pub struct ButtonEvent {
	pub time: DateTime<Utc>,
	pub name: String,
	pub state: bool
}

impl super::ToRecord for ButtonEvent {
	fn fill(&self, record: &mut influxc::Record) {
		record.measurement("button")
			.timestamp(self.time)
			.field("name", Value::from(self.name.clone()))
			.field("state", Value::from(self.state));
	}
}
