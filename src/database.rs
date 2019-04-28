extern crate rusqlite;

use rusqlite::{Connection, Error, NO_PARAMS};
use rusqlite::types::ToSql;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::weather::WeatherEvent;

pub struct Database {
	conn: Connection
}

fn to_seconds(t: &SystemTime) -> u32 {
	t.duration_since(UNIX_EPOCH).unwrap().as_secs() as u32
}

impl Database {
	pub fn new(path: &Path) -> Result<Self, Error> {
		let conn = Connection::open(&path)?;

		conn.execute(
			"CREATE TABLE IF NOT EXISTS weather (
				time DATETIME PRIMARY_KEY DEFAULT CURRENT_TIMESTAMP,
				temperature NUM,
				humidity NUM,
				pressure NUM
			)",
			NO_PARAMS)?;

		Ok(Database { conn })
	}

	pub fn store_weather(self: &Database, event: &WeatherEvent) -> Result<(), Error> {
		self.conn.execute(
			"INSERT INTO weather (time, temperature, humidity, pressure) VALUES (?1, ?2, ?3, ?4)",
			&[&to_seconds(&event.timestamp) as &ToSql, &event.temperature, &event.humidity, &event.pressure]
		)?;
		println!("stored weather event {:?}", event);
		Ok(())
	}
}
