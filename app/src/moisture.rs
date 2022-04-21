use std::error::Error;
use std::thread;
use std::thread::{JoinHandle, sleep};
use std::time::{Duration, SystemTime};
use std::str::FromStr;
use std::sync::mpsc::Sender;
use mcp3xxx::{AnalogIn, MCPDevice, SharedMCPDevice};
use rustpi_io::gpio::*;

use crate::event::Event;
use crate::settings::{ADCSettings, MoistureSensorSettings};
use crate::time::UnixTime;

pub type Measurement = u16;

const CALIBRATED_WET: Measurement = 100;
const CALIBRATED_DRY: Measurement = 0;
const CALIBRATED_RANGE: Measurement = CALIBRATED_WET - CALIBRATED_DRY;
const SECONDS_BETWEEN_SAMPLES: u64 = 60;


#[derive(Debug)]
pub struct MoistureEvent {
	pub unix_time: UnixTime,
	pub name: String,
	pub value: Measurement
}

impl MoistureEvent {
	pub fn timestamp(&self) -> u32 {
		self.unix_time.timestamp()
	}
}


#[derive(Debug)]
pub struct MoistureSensor {
	thread: Option<JoinHandle<()>>
}

struct Sensor {
	name: String,
	channel: AnalogIn,
	pub min_reading: Measurement,
	pub max_reading: Measurement
}

impl Drop for MoistureSensor {
	fn drop(&mut self) {
		if let Some(thread) = self.thread.take() {
			thread.join().unwrap();
		}
	}
}

impl Sensor {
	fn new(mcp: SharedMCPDevice, settings: &MoistureSensorSettings) -> Result<Self, Box<dyn Error>> {
		let analog = AnalogIn::single(mcp, settings.channel)?;
		Ok(Sensor { 
			name: settings.name.clone(), 
			channel: analog,
			min_reading: settings.min_reading,
			max_reading: settings.max_reading
		})
	}
}

struct Sample<'a> {
	sensor: &'a Sensor,
	data: Vec<Measurement>
}

impl<'a> Sample<'a> {
	fn new(sensor: &'a Sensor) -> Self {
		Sample {
			sensor,
			data: vec![]
		}
	}

	fn collect(&mut self) {
		match self.sensor.channel.read_value() {
			Ok(value) => if value != 0 { self.data.push(value) },
			Err(e) => error!("ERROR! reading moisture sensor {}", e)
		}
	}

	fn mean(&self) -> Option<Measurement> {
		if self.data.len() == 0 {
			None
		} else {
			let total: Measurement = self.data.iter().sum();
			let mean = (total as f64 / self.data.len() as f64) as Measurement;
			Some(mean)
		}
	}
}

fn calibrate(m: Measurement, min: Measurement, max: Measurement) -> Measurement {
	if m <= min {
		CALIBRATED_WET
	} else if m >= max {
		CALIBRATED_DRY
	} else {
		let range = max - min;
		let raw = range - (m - min);
		let scaled = ((raw as f64) / (range as f64) * (CALIBRATED_RANGE as f64)) as Measurement;
		scaled + CALIBRATED_DRY
	}
}

fn collect(enable: &GPIO, samples: &mut Vec<Sample>, period: Duration) {
	let until = SystemTime::now() + period;

	while SystemTime::now() < until {
		enable.set(GPIOData::High).unwrap();
		sleep(Duration::from_millis(20));

		for ref mut sample in &mut *samples {
			sample.collect();
		}

		enable.set(GPIOData::Low).unwrap();
		sleep(Duration::from_secs(SECONDS_BETWEEN_SAMPLES));
	}	
}

fn report(samples: Vec<Sample>, channel: &Sender<Event>) {
	for sample in samples {
		match sample.mean().map(|m| calibrate(m, sample.sensor.min_reading, sample.sensor.max_reading)) {
			None => {
				error!("No samples collected for moisture sensor {}", sample.sensor.name);
			},
			Some(value) => {
				send_event(sample.sensor, value, &channel);
			}
		}
	}
}

fn main(mcp: MCPDevice, enable: GPIO, settings: Vec<MoistureSensorSettings>, channel: Sender<Event>, period: Duration) {
	info!("Starting {} moisture sensor(s)", settings.len());
	let shared_mcp = mcp.share();
	let sensors: Vec<Sensor> = settings.iter()
		.map(|sensor| Sensor::new(shared_mcp.clone(), &sensor).unwrap())
		.collect();

	info!("Started {} moisture sensor(s)", sensors.len());
	loop {
		let mut samples: Vec<Sample> = sensors.iter().map(|s| Sample::new(s)).collect();
		collect(&enable, &mut samples, period);
		report(samples, &channel);
	}
}

fn send_event(sensor: &Sensor, value: u16, channel: &Sender<Event>) {
	let event = MoistureEvent { 
		unix_time: UnixTime::now(),
		name: sensor.name.clone(),
		value
	};
	match channel.send(Event::MoistureEvent(event)) {
		Ok(_) => {}
		Err(e) => println!("channel send error {}", e)
	}
}

impl MoistureSensor {
	pub fn new(adc: &ADCSettings, sensors: &Vec<MoistureSensorSettings>, channel: Sender<Event>) -> Result<MoistureSensor, Box<dyn Error>> {
		let device = mcp3xxx::device_from_str(&adc.device)?;
		let device_type = FromStr::from_str(&adc.device_type)?;
		let mcp = MCPDevice::new(device, device_type, adc.chip_select_gpio)?;
		let enable = GPIO::new(adc.enable_gpio, GPIOMode::Write)?;

		let period = Duration::from_secs(adc.update);
		let sensors = sensors.to_vec();
		let thread = thread::Builder::new()
			.name("moisture".to_string())
			.spawn(move || { main(mcp, enable, sensors, channel, period); })?;
		Ok(MoistureSensor { 
			thread: Some(thread)
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_calibrate_min() {
		assert_eq!(CALIBRATED_WET, calibrate(0, 0, 1000));
		assert_eq!(CALIBRATED_WET, calibrate(0, 100, 1000));
		assert_eq!(CALIBRATED_WET, calibrate(449, 450, 1000));
	}

	#[test]
	fn test_calibrate_max() {
		assert_eq!(CALIBRATED_DRY, calibrate(1000, 0, 1000));
		assert_eq!(CALIBRATED_DRY, calibrate(1100, 0, 1000));
	}

	#[test]
	fn test_calibrate_in_range() {
		assert_eq!(25, calibrate(800, 200, 1000));
		assert_eq!(50, calibrate(600, 200, 1000));
		assert_eq!(75, calibrate(400, 200, 1000));
	}
}