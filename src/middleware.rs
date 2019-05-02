extern crate iron;
extern crate iron_json_response as ijr;
extern crate logger;
extern crate router;

use crate::database::Database;

use iron::{typemap, BeforeMiddleware};
use iron::middleware::Handler;
use iron::prelude::*;

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

pub trait DbRequestExtension {
	fn get_database(&self) -> Database;
}

impl <'a, 'b>DbRequestExtension for Request<'a, 'b> {
	fn get_database(&self) -> Database {
		let database = self.extensions.get::<DbMiddleware>().unwrap();
		database.clone()
	}
}

pub fn insert<H: Handler>(handler: H, database: Database) -> impl Handler {
	let (logger_before, logger_after) = logger::Logger::new(None);
	let mut chain = Chain::new(handler);
	chain.link_before(logger_before);
	chain.link_before(DbMiddleware { database });
	chain.link_after(ijr::JsonResponseMiddleware::new());
	chain.link_after(logger_after);
	chain	
}
