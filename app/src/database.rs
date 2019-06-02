extern crate r2d2;
extern crate r2d2_sqlite;
extern crate rusqlite;

use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{Error, params, NO_PARAMS};
use rusqlite::types::ToSql;
use std::ops::Range;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use crate::event::Event;
use crate::weather;
use crate::moisture;

pub struct Database {
	pool: r2d2::Pool<SqliteConnectionManager>
}

pub struct TimePeriod {
	pub start: SystemTime,
	pub end: SystemTime
}

pub type UnixTime = u32;
pub type TimeSeries<T> = Vec<(SystemTime, T)>;

fn to_seconds(t: &SystemTime) -> UnixTime {
	t.duration_since(UNIX_EPOCH).unwrap().as_secs() as UnixTime
}

fn to_system_time(s: UnixTime) -> SystemTime {
	UNIX_EPOCH + Duration::from_secs(s as u64)
}

impl TimePeriod {
	fn start_seconds(&self) -> UnixTime { to_seconds(&self.start) }
	fn end_seconds(&self) -> UnixTime { to_seconds(&self.end) }

	pub fn last_hour() -> TimePeriod {
		let now = SystemTime::now();
		TimePeriod {
			start: now.checked_sub(Duration::from_secs(3600)).unwrap(),
			end: now
		}
	}
}

impl Database {
	pub fn new(path: &Path) -> Result<Self, Error> {
		let manager = SqliteConnectionManager::file(&path);
		let pool = r2d2::Pool::new(manager).unwrap();
		let conn = pool.clone().get().unwrap();

		conn.execute(
			"CREATE TABLE IF NOT EXISTS weather (
				time DATETIME PRIMARY_KEY DEFAULT CURRENT_TIMESTAMP,
				temperature NUM,
				humidity NUM,
				pressure NUM
			)",
			NO_PARAMS)?;

		conn.execute(
			"CREATE TABLE IF NOT EXISTS moisture (
				time DATETIME PRIMARY_KEY DEFAULT CURRENT_TIMESTAMP,
				sensor TEXT,
				value NUM
			)",
			NO_PARAMS)?;

		conn.execute(
			"CREATE TABLE IF NOT EXISTS irrigation (
				valve TEXT,
				start DATETIME,
				end DATETIME
			)",
			NO_PARAMS)?;

		info!("Opened database at {}", path.to_str().unwrap());
		Ok(Database { 
			pool
		})
	}

	pub fn clone(&self) -> Database {
		Database {
			pool: self.pool.clone()
		}
	}

	pub fn store_event(&self, event: &Event) -> Result<(), Error> {
		match event {
			Event::WeatherEvent(w) => self.store_weather(w),
			Event::MoistureEvent(m) => self.store_moisture(m),
			_ => Ok(())
		}?;
		Ok(())
	}

	fn conn(&self) -> r2d2::PooledConnection<SqliteConnectionManager> {
		self.pool.clone().get().unwrap()
	}

	fn store_weather(&self, event: &weather::WeatherEvent) -> Result<(), Error> {
		self.conn().execute(
			"INSERT INTO weather (time, temperature, humidity, pressure) VALUES (?1, ?2, ?3, ?4)",
			&[&to_seconds(&event.timestamp) as &ToSql, &event.temperature, &event.humidity, &event.pressure]
		)?;
		Ok(())
	}

	fn store_moisture(&self, event: &moisture::MoistureEvent) -> Result<(), Error> {
		self.conn().execute(
			"INSERT INTO moisture (time, sensor, value) VALUES (?1, ?2, ?3)",
			&[&to_seconds(&event.timestamp) as &ToSql, &event.name, &event.value]
		)?;
		Ok(())
	}

	pub fn store_irrigation(&self, valve: &str, start: SystemTime, end: SystemTime) -> Result<(), Error> {
		self.conn().execute(
			"INSERT INTO irrigation (valve, start, end) VALUES (?1, ?2, ?3)",
			&[&valve.to_string(), &to_seconds(&start) as &ToSql, &to_seconds(&end) as &ToSql]
		)?;
		Ok(())
	}

	pub fn get_latest_weather(&self) -> Result<weather::WeatherEvent, Error> {
		let conn = self.conn();
		let mut stmt = conn.prepare(
			"SELECT time, temperature, humidity, pressure FROM weather ORDER BY time DESC LIMIT 1")?;
		stmt.query_row(params![], |row| {
			Ok(weather::WeatherEvent {
				timestamp: to_system_time(row.get(0)?),
				temperature: row.get(1)?,
				humidity: row.get(2)?,
				pressure: row.get(3)?
			})
		})
	}

	pub fn get_weather_history(&self, period: &TimePeriod) -> Result<Vec<weather::WeatherEvent>, Error> {
		let conn = self.conn();
		let mut stmt = conn.prepare(
			&format!("SELECT time, temperature, humidity, pressure FROM weather WHERE ?1 <= time AND time < ?2 ORDER BY time ASC")
		)?;
		let iter = stmt.query_map(params![period.start_seconds(), period.end_seconds()], |row| {
			Ok( weather::WeatherEvent { 
				timestamp: to_system_time(row.get(0)?),
				temperature: row.get(1)?,
				humidity: row.get(2)?,
				pressure: row.get(3)?
			})
		})?;
		iter.collect()				
	}

	pub fn get_moisture_sensors(&self) -> Result<Vec<String>, Error> {
		let conn = self.conn();
		let mut stmt = conn.prepare("SELECT DISTINCT sensor FROM moisture")?;
		let iter = stmt.query_map(NO_PARAMS, |row| Ok(row.get(0)?))?;
		iter.collect()
	}

	pub fn get_moisture_history(&self, sensor: &str, period: &TimePeriod) -> Result<TimeSeries<moisture::Measurement>, Error> {
		let conn = self.conn();
		let mut stmt = conn.prepare(
			"SELECT time, value FROM moisture WHERE sensor == ?1 AND ?2 <= time AND time < ?3 ORDER BY time ASC"
		)?;
		let iter = stmt.query_map(params![&sensor, period.start_seconds(), period.end_seconds()], |row| {
			Ok( (to_system_time(row.get(0)?), row.get(1)?) )
		})?;
		iter.collect()
	}

	pub fn get_irrigation_history(&self, valve: &str, period: &TimePeriod) -> Result<TimeSeries<Duration>, Error> {
		let conn = self.conn();
		let mut stmt = conn.prepare(
			"SELECT start, end FROM irrigation WHERE valve = ?1 AND ?2 <= start AND end < ?3 ORDER BY start ASC"
		)?;
		let iter = stmt.query_map(params![&valve, period.start_seconds(), period.end_seconds()], |row| {
			let start: UnixTime = row.get(0)?;
			let end: UnixTime = row.get(1)?;
			Ok( (to_system_time(start), Duration::from_secs((end - start) as u64)) )
		})?;
		iter.collect()
	}

	pub fn get_moisture_range_since_last_irrigation(&self, sensor: &str, valve: &str) -> Result<Range<moisture::Measurement>, Error> {
		let conn = self.conn();
		let mut stmt = conn.prepare(
			"SELECT MIN(value), MAX(value) FROM moisture JOIN irrigation 
				WHERE moisture.sensor = ?1 
				  AND irrigation.valve = ?2
				  AND moisture.time > irrigation.end",
		)?;
		stmt.query_row(params![&sensor, &valve], |row| {
			let min: moisture::Measurement = row.get(0)?;
			let max: moisture::Measurement = row.get(1)?;
			Ok(min..max)
		})
	}
}
