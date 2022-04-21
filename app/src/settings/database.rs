#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct DatabaseSettings {
	pub host: String,
    pub port: u16
}
