#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct DatabaseSettings {
	pub url: String,
    pub database: String,
    pub username: String,
    pub password: String
}
