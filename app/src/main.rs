#[macro_use]
extern crate log;
extern crate env_logger;

extern crate pirrigator;

use pirrigator::settings::Settings;
use pirrigator::pirrigator::Pirrigator;

fn main() {
	env_logger::init();

	let s = Settings::new()
		.expect("Unable to load settings");

	debug!("settings: {:?}", s);

	let p = Pirrigator::new(s)
		.expect("Failed to start Pirrigator");

	p.run();
}
