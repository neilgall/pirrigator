extern crate iron;
extern crate mount;

mod api;
mod camera;
mod error;
mod json;
mod middleware;
mod ui;

use crate::database::Database;
use error::bad_request;
use iron::prelude::*;
use mount::Mount;
use router::Router;
use std::str::FromStr;

pub fn get_param<T: FromStr>(req: &Request, name: &str) -> IronResult<T> {
	let param = req.extensions.get::<Router>()
		.and_then(|params| { params.find(name) })
		.ok_or(bad_request(&format!("parameter {} missing", name)))?;
	let value = param.parse().map_err(|_| { bad_request(&format!("cannot parse parameter {}", name)) })?;
	Ok(value)
}

pub fn run(database: Database) {
	let mut mount = Mount::new();
	mount.mount("/api", api::api());
	mount.mount("/camera", camera::api());
	mount.mount("/", ui::ui());

	Iron::new(middleware::insert(mount, database))
		.http("0.0.0.0:5000")
		.unwrap();
}
