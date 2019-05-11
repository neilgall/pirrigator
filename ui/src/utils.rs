extern crate chrono;

use chrono::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

pub const HOUR: u32 = 3600;
pub const DAY: u32 = 86400;
pub const WEEK: u32 = 86400*7;
pub const MONTH: u32 = 86400*30;

pub fn render_system_time(system_time: SystemTime) -> String {
    let unixtime = system_time.duration_since(UNIX_EPOCH).unwrap();
    Utc.timestamp(unixtime.as_secs() as i64, unixtime.subsec_nanos()).to_rfc2822()
}