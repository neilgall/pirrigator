extern crate bme280;

use bme280::Bme280Device;

pub fn read(device: &str, address: u16) {
	let mut dev = Bme280Device::new(device, address)
		.expect("unable to open device");

	let r = dev.read()
		.expect("unable to read data from device");
		
	println!("temperature: {}", r.temperature);
	println!("humidity: {}", r.humidity);
	println!("pressure: {}", r.pressure);
}
