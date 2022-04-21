#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct ValveSettings {
	pub name: String,
	pub socket: String,
	pub gpio: u8
}
