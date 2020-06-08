use crate::button;
use common::moisture;
use common::weather;

#[derive(Debug)]
pub enum Event {
	WeatherEvent(weather::WeatherEvent),
	MoistureEvent(moisture::MoistureEvent),
	ButtonEvent(button::ButtonEvent),
	IrrigateEvent(String)
}

