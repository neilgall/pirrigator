extern crate i2cdev;

use i2cdev::core::*;
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};
use std::error::Error;
use std::thread;
use std::time::Duration;

pub const DEFAULT_DEVICE: &str = "/dev/i2c-1";
pub const DEFAULT_ADDRESS: u16 = 0x76;

const BME280_REG_DATA: u8 = 0xF7;
const BME280_REG_CONTROL: u8 = 0xF4;
const BME280_REG_CONTROL_HUMIDITY: u8 = 0xF2;

const BME280_OVERSAMPLE_TEMPERATURE: u8 = 2 << 5;
const BME280_OVERSAMPLE_PRESSURE: u8 = 2 << 2;
const BME280_OVERSAMPLE_HUMIDITY: u8 = 2;
const BME280_MODE: u8 = 1;

pub struct Bme280Device {
	dev: LinuxI2CDevice
}

#[derive(Debug)]
pub struct Bme280Data {
	pub temperature: f64,
	pub humidity: f64,
	pub pressure: f64
}

fn get_i8(data: &Vec<u8>, index: usize) -> i8 {
	let i = data[index] as i16;
	if i > 127 {
		(i - 256) as i8
	} else {
		i as i8
	}
}

fn get_u16(data: &Vec<u8>, index: usize) -> u16 {
	((data[index+1] as u16) << 8) | data[index] as u16
}

fn get_i16(data: &Vec<u8>, index: usize) -> i16 {
	get_u16(data, index) as i16
}

fn compensate_temperature(cal: &Vec<u8>, data: &Vec<u8>) -> f64 {
	let dig_t1 = get_u16(&cal, 0) as f64;
	let dig_t2 = get_i16(&cal, 2) as f64;
	let dig_t3 = get_i16(&cal, 4) as f64;

	let t_raw = (((data[3] as u32) << 12) | ((data[4] as u32) << 4) | ((data[5] as u32) >> 4)) as f64;
	let var1 = (t_raw / 16384.0 - dig_t1 / 1024.0) * dig_t2;
	let var2 = (t_raw / 131072.0 - dig_t1 / 8192.0).powf(2.0) * dig_t3;
	return var1 + var2;
}

fn compensate_pressure(cal: &Vec<u8>, data: &Vec<u8>, t_fine: f64) -> f64 {
	let dig_p1 = get_u16(&cal, 6) as f64;
	let dig_p2 = get_i16(&cal, 8) as f64;
	let dig_p3 = get_i16(&cal, 10) as f64;
	let dig_p4 = get_i16(&cal, 12) as f64;
	let dig_p5 = get_i16(&cal, 14) as f64;
	let dig_p6 = get_i16(&cal, 16) as f64;
	let dig_p7 = get_i16(&cal, 18) as f64;
	let dig_p8 = get_i16(&cal, 20) as f64;
	let dig_p9 = get_i16(&cal, 22) as f64;

	let p_raw = (((data[0] as u32) << 12) | ((data[1] as u32) << 4) | ((data[2] as u32) >> 4)) as f64;
	let var1 = (t_fine / 2.0) - 64000.0;
	let var2 = var1 * var1 * dig_p6 / 32768.0;
	let var2 = var2 + (var1 * dig_p5 * 2.0);
	let var2 = (var2 / 4.0) + (dig_p4 * 65536.0);
	let var1 = (dig_p3 * var1 * var1 / 524288.0 + dig_p2 * var1) / 524288.0;
	let var1 = (1.0 + var1 / 32768.0) * dig_p1;
	if var1 == 0.0 {
		0.0
	} else {
		let p = 1048576.0 - p_raw;
		let p = (p - (var2 / 4096.0)) * 6250.0 / var1;
		let var1 = dig_p9 * p * p / 2147483648.0;
		let var2 = p * dig_p8 / 32768.0;
		p + (var1 + var2 + dig_p7) / 16.0
	}
}

fn compensate_humidity(cal2: &Vec<u8>, cal3: &Vec<u8>, data: &Vec<u8>, t_fine: f64) -> f64 {
	let dig_h1 = get_i8(cal2, 0) as f64;
	let dig_h2 = get_i16(&cal3, 0) as f64;
	let dig_h3 = cal3[2] as f64;
	let dig_h4 = (((get_i8(&cal3, 3) as i16) << 4) | (cal3[4] & 0x0F) as i16) as f64;
	let dig_h5 = (((get_i8(&cal3, 5) as i16) << 4) | (cal3[4] >> 4) as i16) as f64;
	let dig_h6 = get_i8(&cal3, 6) as f64;

	let h_raw = (((data[6] as u16) << 8) | (data[7] as u16)) as f64;
	let var_h = t_fine - 76800.0;
	let var_h = (h_raw - (dig_h4 * 64.0 + dig_h5 / 16384.0 * var_h)) *
					(dig_h2 / 65536.0 * (1.0 + dig_h6 / 67108864.0 * var_h *
						(1.0 + dig_h3 / 67108864.0 * var_h)));
	let var_h = var_h * (1.0 - dig_h1 * var_h / 524288.0);
	if var_h > 100.0 {
		100.0
	} else if var_h < 0.0 {
		0.0
	} else {
		var_h
	}
}

impl Bme280Device {
	pub fn new(device: &str, address: u16) -> Result<Bme280Device, Box<Error>> {
		let dev = LinuxI2CDevice::new(device, address)?;
		Ok(Bme280Device { dev })
	}

	pub fn read(&mut self) -> Result<Bme280Data, LinuxI2CError> {
		self.dev.smbus_write_byte_data(
			BME280_REG_CONTROL_HUMIDITY,
			BME280_OVERSAMPLE_HUMIDITY)?;

		self.dev.smbus_write_byte_data(
			BME280_REG_CONTROL, 
			BME280_OVERSAMPLE_TEMPERATURE 
			| BME280_OVERSAMPLE_PRESSURE
			| BME280_MODE)?;

		let cal1 = self.dev.smbus_read_i2c_block_data(0x88, 24u8)?;
		let cal2 = self.dev.smbus_read_i2c_block_data(0xA1, 1u8)?;
		let cal3 = self.dev.smbus_read_i2c_block_data(0xE1, 7u8)?;


		let wait_time_ms = 1.25 + (2.3 * BME280_OVERSAMPLE_TEMPERATURE as f32)
						  	 	+ ((2.3 * BME280_OVERSAMPLE_PRESSURE as f32) + 0.575)
							 	+ ((2.3 * BME280_OVERSAMPLE_HUMIDITY as f32) + 0.575);
		thread::sleep(Duration::from_micros((wait_time_ms * 1000.0) as u64));

		let data = self.dev.smbus_read_i2c_block_data(BME280_REG_DATA, 8)?;

		let t_fine = compensate_temperature(&cal1, &data);
		let pressure = compensate_pressure(&cal1, &data, t_fine);
		let humidity = compensate_humidity(&cal2, &cal3, &data, t_fine);	

		return Ok(Bme280Data { 
			temperature: t_fine / 5120.0,
			pressure: pressure / 100.0,
			humidity
		});
	}
}
