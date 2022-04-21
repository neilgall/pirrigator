use crate::event::moisture::Measurement;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct ADCSettings {
	pub device: String,
	pub device_type: String,
	pub chip_select_gpio: u8,
	pub enable_gpio: u8,
	pub update: u64
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct MoistureSensorSettings {
	pub name: String,
	pub socket: String,
	pub channel: u8,
	pub min_reading: Measurement,
	pub max_reading: Measurement
}
