use influxdb::InfluxDbWriteable;
use chrono::{DateTime, Utc};

#[derive(Debug, InfluxDbWriteable)]
pub struct ButtonEvent {
	pub time: DateTime<Utc>,
	pub name: String,
	pub state: bool
}
