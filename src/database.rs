extern crate r2d2;
extern crate r2d2_sqlite;
extern crate rusqlite;

use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{Error, params, NO_PARAMS};
use rusqlite::types::ToSql;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use crate::event::Event;
use crate::weather;
use crate::moisture;

pub struct Database {
	pool: r2d2::Pool<SqliteConnectionManager>
}

fn to_seconds(t: &SystemTime) -> u32 {
	t.duration_since(UNIX_EPOCH).unwrap().as_secs() as u32
}

fn to_system_time(s: u32) -> SystemTime {
	UNIX_EPOCH + Duration::from_secs(s as u64)
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

		println!("Opened database at {}", path.to_str().unwrap());
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
}
