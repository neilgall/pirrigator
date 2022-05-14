use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct ButtonEvent {
	pub time: DateTime<Utc>,
	pub name: String,
	pub state: bool
}

impl super::ToInfluxDB for ButtonEvent {
	fn to_line(&self) -> String {
		format!("button,name={} state={} {}",
				self.name,
				self.state,
				self.time.timestamp()
		)
	}
}
