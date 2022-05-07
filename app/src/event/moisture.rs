use chrono::{DateTime, Utc};
use influxdb::InfluxDbWriteable;

pub type Measurement = u16;

#[derive(Debug, Clone, InfluxDbWriteable)]
pub struct MoistureEvent {
	pub time: DateTime<Utc>,
	pub name: String,
	pub value: Measurement
}

