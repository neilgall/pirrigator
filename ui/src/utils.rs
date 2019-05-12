extern crate chrono;

use chrono::prelude::*;
use std::f64;
use std::time::{SystemTime, UNIX_EPOCH};

pub const HOUR: u32 = 3600;
pub const DAY: u32 = 86400;
pub const WEEK: u32 = 86400*7;
pub const MONTH: u32 = 86400*30;

pub fn to_utc(system_time: &SystemTime) -> DateTime<Utc> {
    let unixtime = system_time.duration_since(UNIX_EPOCH).unwrap();
    Utc.timestamp(unixtime.as_secs() as i64, unixtime.subsec_nanos())
}

pub fn render_system_time(system_time: &SystemTime) -> String {
	to_utc(system_time).to_rfc2822()
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
