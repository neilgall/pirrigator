use chrono::{DateTime, Utc};
use influxc::Value;

#[derive(Debug)]
pub struct IrrigatedEvent {
    pub time: DateTime<Utc>,
    pub name: String,
    pub seconds: u32
}

impl super::ToRecord for IrrigatedEvent {
    fn fill(&self, record: &mut influxc::Record) {
        record.measurement("irrigated")
            .timestamp(self.time)
            .field("name", Value::from(self.name.clone()))
            .field("durationSeconds", Value::from(self.seconds as i64));
    }
}