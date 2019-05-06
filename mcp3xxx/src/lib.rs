extern crate rustpi_io;

use rustpi_io::*;
use rustpi_io::gpio::*;
use rustpi_io::serial::Device;
use std::io::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;

pub enum MCPDeviceType {
	MCP3008,
	MCP3004
}

pub struct MCPDevice {
	dev: serial::SerialPi,
	mcp_type: MCPDeviceType,
	chip_select: GPIO
}

pub type SharedMCPDevice = Rc<RefCell<MCPDevice>>;

pub struct AnalogIn {
	mcp: SharedMCPDevice,
	pin: u8,
	command: u8
}

pub fn device_from_str(s: &str) -> Result<rustpi_io::serial::Device> {
	match s {
		"CE0" => Ok(rustpi_io::serial::Device::CE0),
		"CE1" => Ok(rustpi_io::serial::Device::CE1),
		_ => Err(Error::new(ErrorKind::NotFound, "unknown device"))
	}
}

impl FromStr for MCPDeviceType {
	type Err = Error;
	fn from_str(s: &str) -> Result<Self> {
		match s {
			"MCP3004" => Ok(MCPDeviceType::MCP3004),
			"MCP3008" => Ok(MCPDeviceType::MCP3008),
			_ => Err(Error::new(ErrorKind::NotFound, "unknown device type"))
		}
	}
}

impl MCPDevice {
	pub fn new(device: Device, mcp_type: MCPDeviceType, chip_select_pin: u8) -> Result<MCPDevice> {
		let mut dev = serial::SerialPi::new(
			device,
			serial::Speed::Khz122,
			serial::SpiMode::Mode0,
			serial::ComMode::FullDuplex)?;
		dev.try_shrink_to(3);
		let chip_select = GPIO::new(chip_select_pin, GPIOMode::Write)?;
		return Ok(MCPDevice { dev, mcp_type, chip_select });
	}

	pub fn validate_pin(&self, pin: u8) -> Result<()> {
		let max_pin = match &self.mcp_type {
			MCPDeviceType::MCP3004 => 3,
			MCPDeviceType::MCP3008 => 7
		};
		if pin > max_pin {
			Err(Error::new(ErrorKind::InvalidInput, "pin is invalid"))
		} else {
			Ok(())
		}
	}

	fn differential_channel(&self, pin: u8, neg_pin: u8) -> Result<u8> {
		match &self.mcp_type {
			MCPDeviceType::MCP3004 => match (pin, neg_pin) {
				(0, 1) => Ok(0),
				(1, 0) => Ok(1),
				(2, 3) => Ok(2),
				(3, 2) => Ok(3),
				_ => Err(Error::new(ErrorKind::InvalidInput, "invalid pin pair"))
			},
			MCPDeviceType::MCP3008 => match (pin, neg_pin) {
				(0, 1) => Ok(0),
				(1, 0) => Ok(1),
				(2, 3) => Ok(2),
				(3, 2) => Ok(3),
				(4, 5) => Ok(4),
				(5, 4) => Ok(5),
				(6, 7) => Ok(6),
				(7, 6) => Ok(7),
				_ => Err(Error::new(ErrorKind::InvalidInput, "invalid pin pair"))
			}
		}
	}

	pub fn share(self) -> SharedMCPDevice {
		Rc::new(RefCell::new(self))
	}
}

impl AnalogIn {
	pub fn single(mcp: Rc<RefCell<MCPDevice>>, pin: u8) -> Result<Self> {
		mcp.borrow().validate_pin(pin)?;
		Ok(AnalogIn { mcp: mcp.clone(), pin, command: 0x03 })
	}

	pub fn differential(mcp: Rc<RefCell<MCPDevice>>, pin: u8, neg_pin: u8) -> Result<AnalogIn> {
		let pin = mcp.borrow().differential_channel(pin, neg_pin)?;
		Ok(AnalogIn { mcp: mcp.clone(), pin, command: 0x02 })
	}

	pub fn read_value(&self) -> Result<u16> {
		let command = (self.command << 6) | (self.pin << 3);
		let out_buf: [u8; 3] = [command, 0, 0];
		let mut in_buf: [u8; 3] = [0, 0, 0];

		let mut mcp = self.mcp.borrow_mut();

		mcp.chip_select.set(GPIOData::Low)?;
		mcp.dev.write(&out_buf)?;
		mcp.dev.read(&mut in_buf)?;
		mcp.chip_select.set(GPIOData::High)?;

		let value = (((in_buf[0] & 0x01) as u16) << 9)
					| ((in_buf[1] as u16) << 1)
					| (in_buf[2] >> 7) as u16;
		Ok(value)
	}

	pub fn read_voltage(&mut self) -> Result<f64> {
		let value = self.read_value()?;
		Ok((value as f64 * 3.3) / 65535.0)
	}
}