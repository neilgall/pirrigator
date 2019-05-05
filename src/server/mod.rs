extern crate iron;
extern crate iron_json_response as ijr;
extern crate mount;
extern crate router;

mod api;
mod error;
mod json;

use crate::database::Database;
use crate::middleware;

use iron::prelude::*;
use mount::Mount;

// const index_html: [u8] = include_bytes!("../draco-starter/index.html");
// const draco_starter_js: [u8] = include_bytes!("../draco-starter/build/draco-starter.js");
// const draco_starter_wasm: [u8] = include_bytes!("../draco-starter/build/draco-starter_bg.wasm");

pub fn run(database: Database) {
	let mut mount = Mount::new();
	mount.mount("/api", api::api());

	Iron::new(middleware::insert(mount, database))
		.http("0.0.0.0:5000")
		.unwrap();
}
