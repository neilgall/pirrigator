extern crate iron;
extern crate mount;
extern crate urlencoding;

mod api;
mod error;
mod json;
mod middleware;
mod ui;

use crate::database::Database;
use crate::event::Event;
use crate::server::middleware::PirrigatorData;

use error::bad_request;
use iron::prelude::*;
use mount::Mount;
use router::Router;
use std::str::FromStr;
use std::sync::mpsc::Sender;
use crate::controller::Zone;

fn urldecode(s: &str) -> IronResult<String> {
	urlencoding::decode(s)
		.map_err(|_| bad_request(&format!("cannot urldecode {}", s)))
}

fn parse<T: FromStr>(s: String) -> IronResult<T> {
	s.parse()
		.map_err(|_| bad_request(&format!("cannot parse parameter {}", s)))
}

pub fn get_param<T: FromStr>(req: &Request, name: &str) -> IronResult<T> {
	req.extensions.get::<Router>()
		.and_then(|params| params.find(name))
		.ok_or_else(|| bad_request(&format!("parameter {} missing", name)))
		.and_then(urldecode)
		.and_then(parse)
}

pub fn run(database: Database, zones: &Vec<Zone>, tx: Sender<Event>) {
	let mut mount = Mount::new();
	mount.mount("/api", api::api());
	mount.mount("/", ui::ui());

	let data = PirrigatorData::new(database, zones, tx);
	Iron::new(middleware::insert(mount, data))
		.http("0.0.0.0:5000")
		.unwrap();
}
