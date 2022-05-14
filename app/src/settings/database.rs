#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct DatabaseSettings {
	pub url: String,
    pub token: String,
    pub backlog_path: Option<String>,
    pub organisation: String,
    pub bucket: String
}
