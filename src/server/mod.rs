extern crate iron;
extern crate iron_json_response as ijr;
extern crate mount;
mod api;
mod error;
mod json;
mod ui;

use crate::database::Database;
use crate::middleware;

use iron::prelude::*;
use mount::Mount;

pub fn run(database: Database) {
	let mut mount = Mount::new();
	mount.mount("/api", api::api());
	mount.mount("/", ui::ui());

	Iron::new(middleware::insert(mount, database))
		.http("0.0.0.0:5000")
		.unwrap();
}
