use std::error::Error;
use std::thread::{JoinHandle, spawn, sleep};
use std::time::{Duration, SystemTime};
use std::str::FromStr;
use std::sync::mpsc::Sender;
use mcp3xxx::{MCPDevice, AnalogIn};
use crate::event::Event;


#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct ADCSettings {
	pub device: String,
	pub device_type: String,
	pub enable_gpio: u8,
	pub update: u64
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct MoistureSensorSettings {
	pub name: String,
	pub channel: u8
}

#[derive(Debug)]
pub struct MoistureSensor {
	thread: JoinHandle<()>
}

#[derive(Debug)]
pub struct MoistureEvent {
	pub timestamp: SystemTime,
	pub name: String,
	pub value: u16
}

struct Sensor {
	name: String,
	channel: AnalogIn
}

impl Sensor {
	fn new(mcp: &MCPDevice, settings: &MoistureSensorSettings) -> Result<Self, Box<Error>> {
		let analog = mcp.single_analog_in(settings.channel)?;
		Ok(Sensor { name: settings.name.clone(), channel: analog })
	}
}

fn main(mcp: &mut MCPDevice, sensors: Vec<Sensor>, channel: Sender<Event>, period: Duration) {
	loop {
		for sensor in &sensors {
			match mcp.read_value(&sensor.channel) {
				Ok(value) => send_event(&sensor, value, &channel),
				Err(e) => println!("ERROR! reading moisture sensor {}", e)
			}
		}
		sleep(period);
	}
}

fn send_event(sensor: &Sensor, value: u16, channel: &Sender<Event>) {
	let event = MoistureEvent { 
		timestamp: SystemTime::now(),
		name: sensor.name.clone(),
		value
	};
	match channel.send(Event::MoistureEvent(event)) {
		Ok(_) => {}
		Err(e) => println!("channel send error {}", e)
	}
}

impl MoistureSensor {
	pub fn new(adc: &ADCSettings, sensors: &Vec<MoistureSensorSettings>, channel: Sender<Event>) -> Result<MoistureSensor, Box<Error>> {
		let device = mcp3xxx::device_from_str(&adc.device)?;
		let device_type = FromStr::from_str(&adc.device_type)?;
		let mut mcp = MCPDevice::new(device, device_type, adc.enable_gpio)?;

		let analogs = sensors.iter()
			.map(|sensor| Sensor::new(&mcp, &sensor).unwrap())
			.collect();

		let period = Duration::from_secs(adc.update);
		let thread = spawn(move || main(&mut mcp, analogs, channel, period));

		Ok(MoistureSensor { thread })
	}
}
