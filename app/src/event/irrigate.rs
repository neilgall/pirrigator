use chrono::{DateTime, Utc};
use influxdb::InfluxDbWriteable;

#[derive(Debug, Clone, InfluxDbWriteable)]
pub struct IrrigatedEvent {
    pub time: DateTime<Utc>,
    pub name: String,
    pub seconds: u32
}
