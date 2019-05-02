extern crate iron;
extern crate router;

use crate::database::Database;

use iron::{typemap, BeforeMiddleware};
use iron::prelude::*;
use iron::status;
use router::Router;

struct DbMiddleware {
	database: Database
}

impl typemap::Key for DbMiddleware {
	type Value = Database;
}

impl BeforeMiddleware for DbMiddleware {
	fn before(&self, req: &mut Request) -> IronResult<()> {
		req.extensions.insert::<DbMiddleware>(self.database.clone());
		Ok(())
	}
}

trait DbRequestExtension {
	fn get_db_conn(&self) -> Database;
}

impl <'a, 'b>DbRequestExtension for Request<'a, 'b> {
	fn get_db_conn(&self) -> Database {
		let database = self.extensions.get::<DbMiddleware>().unwrap();
		database.clone()
	}
}

fn root(_: &mut Request) -> IronResult<Response> {
	Ok(Response::with((status::Ok, "Hello World!")))
}

pub fn run(database: Database) {
	let mut router = Router::new();
	router.get("/", root, "root");

	let mut chain = Chain::new(router);
	chain.link_before(DbMiddleware { database });

	Iron::new(chain).http("localhost:5000").unwrap();
}
