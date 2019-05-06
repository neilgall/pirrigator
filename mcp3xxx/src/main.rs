extern crate mcp3xxx;

use std::thread;
use std::time::Duration;
use mcp3xxx::*;

fn main() {
	let mcp = MCPDevice::new(rustpi_io::serial::Device::CE0, MCPDeviceType::MCP3008, 22)
		.expect("can't open device");
	let mcp = mcp.share();

	let ch0 = AnalogIn::single(mcp.clone(), 0)
		.expect("can't get analog in channel 0");

	let ch1 = AnalogIn::single(mcp.clone(), 1)
		.expect("can't get analog in channel 1");

	let ch2 = AnalogIn::single(mcp.clone(), 2)
		.expect("can't get analog in channel 2");

	loop {
		let r0 = ch0.read_value().expect("can't read value from channel 0");
		let r1 = ch1.read_value().expect("can't read value from channel 1");
		let r2 = ch2.read_value().expect("can't read value from channel 2");
		println!("{} {} {}", r0, r1, r2);
		thread::sleep(Duration::from_millis(500));
	}
}