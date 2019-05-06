extern crate bme280;

use bme280::{Bme280Device, DEFAULT_DEVICE, DEFAULT_ADDRESS};

fn main() {
	let mut dev = Bme280Device::new(DEFAULT_DEVICE, DEFAULT_ADDRESS)?;

	let r = dev.read_all()?;
	println!("temperature: {}", r.temperature);
	println!("humidity: {}", r.humidity);
	println!("pressure: {}", r.pressure);
}
