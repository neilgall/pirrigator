use chrono::{DateTime, Utc};
use influxdb::InfluxDbWriteable;

#[derive(Debug, InfluxDbWriteable)]
pub struct IrrigatedEvent {
    pub time: DateTime<Utc>,
    pub name: String,
    pub seconds: u32
}
