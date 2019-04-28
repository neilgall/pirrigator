use crate::weather;

pub enum Event {
	WeatherEvent(weather::WeatherEvent)
}

