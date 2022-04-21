#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct WeatherSensorSettings {
	pub device: String,
	pub address: u16,
	pub update: u64
}
