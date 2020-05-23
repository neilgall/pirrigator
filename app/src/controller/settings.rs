use common::moisture::Measurement;


#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Check {
	pub start: String,
	pub every: String,
	pub duration: String
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Zone {
	pub name: String,
	pub valve: String,
	pub sensors: Vec<String>,
	pub threshold: Measurement,
	pub check: Vec<Check>,
	pub irrigate_seconds: u64
}

#[derive(Clone, Debug, Deserialize)]
pub struct Location {
	pub latitude: f64,
	pub longitude: f64
}

impl PartialEq for Location {
	fn eq(&self, other: &Location) -> bool {
		(self.latitude - other.latitude).abs() < std::f64::EPSILON &&
		(self.longitude - other.longitude).abs() < std::f64::EPSILON
	}
}

impl Eq for Location {}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct ControllerSettings {
	pub location: Location,
	pub zones: Vec<Zone>
}


