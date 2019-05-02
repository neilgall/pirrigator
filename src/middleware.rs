extern crate iron;
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

trait DbRequestExtension {
	fn get_db_conn(&self) -> Database;
}

impl <'a, 'b>DbRequestExtension for Request<'a, 'b> {
	fn get_db_conn(&self) -> Database {
		let database = self.extensions.get::<DbMiddleware>().unwrap();
		database.clone()
	}
}

pub fn insert<H: Handler>(handler: H, database: Database) -> impl Handler {
	let mut chain = Chain::new(handler);
	chain.link_before(DbMiddleware { database });
	chain	
}
