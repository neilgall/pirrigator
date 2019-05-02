extern crate iron;
extern crate router;

use crate::database::Database;
use crate::middleware;

use iron::prelude::*;
use iron::status;
use router::Router;

fn root(_: &mut Request) -> IronResult<Response> {
	Ok(Response::with((status::Ok, "Hello World!")))
}

pub fn run(database: Database) {
	let mut router = Router::new();
	router.get("/", root, "root");

	Iron::new(middleware::insert(router, database))
		.http("localhost:5000")
		.unwrap();
}
