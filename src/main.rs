extern crate pirrigator;

use pirrigator::settings::Settings;
use pirrigator::Pirrigator;

fn main() {
	let s = Settings::new()
		.expect("Unable to load settings");

	println!("settings: {:?}", s);

	let p = Pirrigator::new(s)
		.expect("Failed to start Pirrigator");

	p.run_server();
}
