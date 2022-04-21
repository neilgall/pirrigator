#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct DatabaseSettings {
	pub url: String,
    pub database: String
}
