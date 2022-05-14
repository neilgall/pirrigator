use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct IrrigatedEvent {
    pub time: DateTime<Utc>,
    pub name: String,
    pub seconds: u32
}

impl super::ToInfluxDB for IrrigatedEvent {
    fn to_line(&self) -> String {
        format!("irrigated,name={} durationSeconds={} {}",
                self.name,
                self.seconds,
                self.time.timestamp()
        )
    }
}