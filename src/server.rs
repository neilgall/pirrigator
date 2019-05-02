extern crate iron;
extern crate iron_json_response as ijr;
extern crate router;

use crate::database::Database;
use crate::middleware;

use iron::prelude::*;
use iron::status;
use ijr::JsonResponse;
use middleware::DbRequestExtension;
use router::Router;

fn status(_: &mut Request) -> IronResult<Response> {
	Ok(Response::with((status::Ok, "running")))
}

fn weather(req: &mut Request) -> IronResult<Response> {
	let weather = req.get_database().get_latest_weather();
	println!("weather {:?}", weather);

	let mut response = Response::new();
	match weather {
		Ok(w) => {
			response.set_mut(JsonResponse::json(w)).set_mut(status::Ok);
			Ok(response)
		}
		Err(e) => {
			let err = IronError::new(e, status::NotFound);
			Err(err)
		}
	}
}

pub fn run(database: Database) {
	let mut router = Router::new();
	router.get("/status", status, "status");
	router.get("/weather", weather, "weather");

	Iron::new(middleware::insert(router, database))
		.http("0.0.0.0:5000")
		.unwrap();
}
