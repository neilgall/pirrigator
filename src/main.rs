#[macro_use]
extern crate serde_derive;

use std::path::Path;

mod database;
mod settings;
mod weather;

fn main() {
	let s = settings::Settings::new()
		.expect("Unable to load settings");

	println!("settings: {:?}", s);

	let _ = database::Database::new(Path::new(&s.database.path))
		.expect("Unable to open database");

	weather::read(&s.weather.device, s.weather.address);
}
