#[macro_use]
extern crate serde_derive;

// extern crate bme280;

// use bme280::{Bme280Device, DEFAULT_DEVICE, DEFAULT_ADDRESS};

mod settings;

fn main() {
	let s = settings::Settings::new().expect("Unable to load settings");

	println!("settings: {:?}", s);

	// let mut dev = Bme280Device::new(DEFAULT_DEVICE, DEFAULT_ADDRESS)
	// 	.expect("unable to open device");

	// let r = dev.read()
	// 	.expect("unable to read data from device");
		
	// println!("temperature: {}", r.temperature);
	// println!("humidity: {}", r.humidity);
	// println!("pressure: {}", r.pressure);
}
