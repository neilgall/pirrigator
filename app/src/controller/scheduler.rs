extern crate chrono;
extern crate sunrise;

use chrono::prelude::*;
use chrono::Duration;
use std::error::Error;
use std::fmt;
use std::str::FromStr;
use std::sync::mpsc::Sender;
use std::thread::{JoinHandle, sleep, spawn};

use crate::event::Event;
use crate::settings::controller::{Location, Zone};

#[derive(Debug)]
pub struct ParseError {
	msg: String
}

impl From<std::num::ParseIntError> for ParseError {
	fn from(e: std::num::ParseIntError) -> Self {
		ParseError { msg: e.to_string() }
	}
}

impl fmt::Display for ParseError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", &self.msg)
	}
}

impl Error for ParseError {}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Time {
	Sunrise,
	Sunset,
	Fixed { hour: u32, minute: u32 }
}

impl FromStr for Time {
	type Err = ParseError;

	fn from_str(s: &str) -> Result<Time, ParseError> {
		match s.to_lowercase().as_ref() {
			"sunrise" => Ok(Time::Sunrise),
			"sunset" => Ok(Time::Sunset),
			_ => {
				let parts: Vec<&str> = s.split(":").collect();
				if parts.len() == 2 {
					let hour: u32 = parts[0].parse()?;
					let minute: u32 = parts[1].parse()?;
					Ok(Time::Fixed { hour, minute })
				} else {
					Err(ParseError { msg: format!("unable to parse time {}", s)})
				}
			}
		}
	}
}

fn parse_duration(s: &str) -> Result<Duration, ParseError> {
	let minutes: i64 = s.parse()?;
	Ok(Duration::seconds(minutes * 60))
}

#[derive(Clone, Debug)]
struct ScheduledEvent {
	name: String,
	time: Time,
	every: Duration,
	duration: Duration
}

impl ScheduledEvent {
	fn new(name: &str, time: &str, every: &str, duration: &str) -> Result<Self, ParseError> {
		let event = ScheduledEvent { 
			name: name.to_string(),
			time: Time::from_str(time)?,
			every: parse_duration(every)?,
			duration: parse_duration(duration)?
		};
		if event.every == Duration::zero() {
			Err(ParseError { msg: "'every' must be greater than zero ".to_string() })
		} else if event.duration == Duration::zero() {
			Err(ParseError { msg: "'duration' must be greater than zero".to_string() })
		} else {
			Ok(event)
		}
	}

	fn times(&self, location: &Location, date: &Date<Utc>) -> Vec<DateTime<Utc>> {
		let (sunrise, sunset) = sunrise::sunrise_sunset(location.latitude, location.longitude, date.year(), date.month(), date.day());
		let start = match self.time {
			Time::Sunrise => Utc.timestamp(sunrise, 0),
			Time::Sunset => Utc.timestamp(sunset, 0),
			Time::Fixed { hour, minute } => date.and_hms(hour, minute, 0)
		};
		let mut dur: Duration = Duration::seconds(0);
		let mut times = vec![];
		while dur < self.duration {
			match start.checked_add_signed(dur) {
				Some(t) => times.push(t),
				None => warn!("time add error!")
			}
			dur = dur + self.every;
		}
		times
	}
}

#[derive(Debug, Eq, PartialEq)]
struct Pending<'a> {
	name: &'a str,
	time: DateTime<Utc>
}

impl<'a> Pending<'a> {
	fn new(name: &'a str, time: &DateTime<Utc>) -> Self {
		Pending { name, time: *time }
	}
}

#[derive(Debug)]
struct Schedule {
	events: Vec<ScheduledEvent>,
	location: Location
}

impl Schedule {
	pub fn new(events: &Vec<ScheduledEvent>, location: &Location) -> Self {
		Schedule { 
			events: events.to_vec(),
			location: location.clone()
		}
	}

	fn from_zones(zones: &Vec<Zone>) -> Result<Vec<ScheduledEvent>, ParseError> {
		let mut events: Vec<ScheduledEvent> = vec![];
		for zone in zones {
			for check in &zone.check {
				events.push(ScheduledEvent::new(&zone.name, &check.start, &check.every, &check.duration)?);
			}
		}
		Ok(events)
	}

	fn all_pending(&self, now: DateTime<Utc>) -> Vec<Pending> {
		let mut events: Vec<Pending> = vec![];
		for event in &self.events {
			let times = event.times(&self.location, &now.date());
			times.iter()
				.filter(|t| *t > &now)
				.for_each(|t| events.push(Pending::new(&event.name, t)));
		}
		events.sort_by(|x, y| x.time.cmp(&y.time) );
		events
	}

	fn main(&self, tx: Sender<Event>) {
		loop {
			let events = self.all_pending(Utc::now());
			if events.is_empty() {
				sleep(std::time::Duration::from_secs(600));
			} else {
				debug!("scheduled {:?}", events);
				for event in events {
					while Utc::now() < event.time {
						sleep(std::time::Duration::from_secs(1));
					}
					tx.send(Event::ConditionalIrrigateEvent(event.name.to_string()))
						.expect("scheduler send error");
				}
			}
		}
	}
}

pub struct Scheduler {
	thread: Option<JoinHandle<()>>
}

impl Drop for Scheduler {
	fn drop(&mut self) {
		if let Some(thread) = self.thread.take() {
			thread.join().unwrap();
		}
	}
}

impl Scheduler {
	pub fn new(location: &Location, zones: &Vec<Zone>, tx: Sender<Event>) -> Result<Self, ParseError> {
		let schedule = Schedule::new(&Schedule::from_zones(zones)?, &location);
		let thread = spawn(move || schedule.main(tx));
		Ok(Scheduler { 
			thread: Some(thread)
		})
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_parse_time() {
		assert_eq!(Time::Sunrise, Time::from_str("SUNRISE").unwrap());
		assert_eq!(Time::Sunrise, Time::from_str("sunrise").unwrap());
		assert_eq!(Time::Sunset, Time::from_str("SunSet").unwrap());
		assert_eq!(Time::Sunset, Time::from_str("sunset").unwrap());
		assert_eq!(Time::Fixed { hour: 6, minute: 19 }, Time::from_str("06:19").unwrap());
		assert!(Time::from_str("foo").is_err());
	}

	#[test]
	fn times_for_schedule_with_fixed_times() {
		let location = Location { longitude: 0.0, latitude: 0.0 };
		let date = Utc.ymd(2019, 5, 19);
		assert_eq!(
			vec![date.and_hms(6, 0, 0), date.and_hms(6, 30, 0), date.and_hms(7, 0, 0), date.and_hms(7, 30, 0)],
			ScheduledEvent::new("test", "06:00", "30", "120").unwrap().times(&location, &date)
		);
	}

	#[test]
	fn times_for_schedule_at_sunrise() {
		let location = Location { longitude: 3.297, latitude: 55.9 };
		let date = Utc.ymd(2019, 5, 19);
		assert_eq!(
			vec![date.and_hms(3, 28, 9), date.and_hms(3, 43, 9), date.and_hms(3, 58, 9), date.and_hms(4, 13, 9)],
			ScheduledEvent::new("test", "sunrise", "15", "60").unwrap().times(&location, &date)
		)
	}

	#[test]
	fn times_for_schedule_at_sunset() {
		let location = Location { longitude: 3.297, latitude: 55.9 };
		let date = Utc.ymd(2019, 5, 19);
		assert_eq!(
			vec![date.and_hms(19, 58, 38), date.and_hms(20, 13, 38), date.and_hms(20, 28, 38), date.and_hms(20, 43, 38)],
			ScheduledEvent::new("test", "sunset", "15", "60").unwrap().times(&location, &date)
		)		
	}

	#[test]
	fn repeat_and_duration_cannot_be_zero() {
		assert!(ScheduledEvent::new("test", "06:00", "30", "0").is_err());
		assert!(ScheduledEvent::new("test", "06:00", "0", "30").is_err());
		assert!(ScheduledEvent::new("test", "06:00", "0", "0").is_err());
	}

	#[test]
	fn times_for_non_repeating_event() {
		let location = Location { longitude: 0.0, latitude: 0.0 };
		let date = Utc.ymd(2019, 5, 19);
		assert_eq!(
			vec![date.and_hms(6, 0, 0)],
			ScheduledEvent::new("test", "06:00", "1", "1").unwrap().times(&location, &date)
		);
	}

	#[test]
	fn schedule_merges_events() {
		let location = Location { longitude: 3.297, latitude: 55.9 };
		let date = Utc.ymd(2019, 5, 19);
		let events = vec![
			ScheduledEvent::new("foo", "08:00", "30", "65").unwrap(),
			ScheduledEvent::new("bar", "08:12", "20", "65").unwrap()
		];
		let schedule = Schedule::new(&events, &location);
		assert_eq!(
			vec![
				Pending::new("foo", &date.and_hms(8, 0, 0)),
				Pending::new("bar", &date.and_hms(8, 12, 0)),
				Pending::new("foo", &date.and_hms(8, 30, 0)),
				Pending::new("bar", &date.and_hms(8, 32, 0)),
				Pending::new("bar", &date.and_hms(8, 52, 0)),
				Pending::new("foo", &date.and_hms(9, 0, 0)),
				Pending::new("bar", &date.and_hms(9, 12, 0))
			],
			schedule.all_pending(date.and_hms(0, 0, 0))
		)
	}

	#[test]
	fn schedule_filters_past_events() {
		let location = Location { longitude: 3.297, latitude: 55.9 };
		let date = Utc.ymd(2019, 5, 19);
		let events = vec![
			ScheduledEvent::new("foo", "08:00", "30", "65").unwrap(),
			ScheduledEvent::new("bar", "08:12", "20", "65").unwrap()
		];
		let schedule = Schedule::new(&events, &location);
		assert_eq!(
			vec![
				Pending::new("bar", &date.and_hms(9, 12, 0))
			],
			schedule.all_pending(date.and_hms(9, 1, 0))
		)
	}

}