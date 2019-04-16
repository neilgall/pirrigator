extern crate rusqlite;

use rusqlite::{Connection, Error, NO_PARAMS};
use std::path::Path;

pub struct Database {
	conn: Connection
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
			NO_PARAMS);

		Ok(Database { conn })
	}
}
