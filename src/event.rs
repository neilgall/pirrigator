use crate::moisture;
use crate::weather;

#[derive(Debug)]
pub enum Event {
	WeatherEvent(weather::WeatherEvent),
	MoistureEvent(moisture::MoistureEvent)
}

