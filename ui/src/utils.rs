extern crate chrono;

use chrono::prelude::*;
use std::f64;
use std::time::{SystemTime, UNIX_EPOCH};

pub const HOUR: u32 = 3600;
pub const HOURS_6: u32 = HOUR * 6;
pub const DAY: u32 = 86400;
pub const DAYS_2: u32 = DAY * 2;
pub const WEEK: u32 = DAY * 7;

pub const SELECTED: &str = "chosen";
pub const UNSELECTED: &str = "not-chosen";
pub const FOREGROUND: &str = "#e7e7e7";
// pub const BACKGROUND: &str = "#1d1f1f";

pub fn to_utc(system_time: &SystemTime) -> DateTime<Utc> {
    let unixtime = system_time.duration_since(UNIX_EPOCH).unwrap();
    Utc.timestamp(unixtime.as_secs() as i64, unixtime.subsec_nanos())
}

pub trait FloatIterExt {
	fn min_value(&mut self) -> f64;
	fn max_value(&mut self) -> f64;
}

impl<T> FloatIterExt for T where T: Iterator<Item=f64> {
	fn min_value(&mut self) -> f64 {
		self.fold(f64::NAN, f64::min)
	}
	fn max_value(&mut self) -> f64 {
		self.fold(f64::NAN, f64::max)
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use std::f64;

	#[test]
	fn test_max_value() {
		let data: Vec<f64> = vec![54.0, 0.0, -34.33, 101.101, 123.4, 99.44, -3.0];
		assert_eq!(123.4, data.iter().cloned().max_value());
	}

	#[test]
	fn test_min_value() {
		let data: Vec<f64> = vec![54.0, 0.0, -34.33, 101.101, 123.4, 99.44, -3.0];
		assert_eq!(-34.33, data.iter().cloned().min_value());
	}

	#[test]
	fn test_max_value_empty() {
		assert!(vec![].iter().cloned().max_value().is_nan());
	}

	#[test]
	fn test_min_value_empty() {
		assert!(vec![].iter().cloned().min_value().is_nan());
	}
}