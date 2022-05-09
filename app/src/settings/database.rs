#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct DatabaseSettings {
	pub url: String,
    pub username: String,
    pub password: String,
    pub backlog_path: String,
    pub organisation: String,
    pub bucket: String
}
