pub mod button;
pub mod irrigate;
pub mod moisture;
pub mod weather;

#[derive(Debug)]
pub enum Event {
	WeatherEvent(weather::WeatherEvent),
	MoistureEvent(moisture::MoistureEvent),
	ButtonEvent(button::ButtonEvent),
	ConditionalIrrigateEvent(String),
	IrrigateEvent(String),
	IrrigatedEvent(irrigate::IrrigatedEvent)
}

