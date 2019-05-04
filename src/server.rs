extern crate iron;
extern crate iron_json_response as ijr;
extern crate router;

use crate::database::{Database, TimePeriod};
use crate::middleware;

use iron::prelude::*;
use iron::status;
use ijr::JsonResponse;
use middleware::DbRequestExtension;
use router::Router;
use serde::ser::Serialize;
use std::error::Error;
use std::str::FromStr;
use std::time::{Duration, SystemTime};

#[derive(Debug)]
struct RequestError {
	msg: String
}

impl RequestError {
	fn new(msg: &str) -> Self {
		RequestError { msg: msg.to_string() }
	}
}

impl std::fmt::Display for RequestError {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}", self.msg)
	}
}

impl Error for RequestError {
	fn description(&self) -> &str {
		&self.msg
	}
}

fn bad_request(msg: &str) -> IronError {
	IronError::new(RequestError::new(msg), status::BadRequest)
}

fn get_param<T: FromStr>(req: &Request, name: &str) -> IronResult<T> {
	let param = req.extensions.get::<Router>()
		.and_then(|params| { params.find(name) })
		.ok_or(bad_request(&format!("parameter {} missing", name)))?;
	let value = param.parse().map_err(|_| { bad_request(&format!("cannot parse parameter {}", name)) })?;
	Ok(value)
}

fn get_time_period(req: &Request) -> IronResult<TimePeriod> {
	let now = SystemTime::now();
	let start = now - Duration::from_secs(get_param(req, "start")?);
	let end = now - Duration::from_secs(get_param(req, "end")?);
	Ok(TimePeriod { start, end })
}

fn json_response<T: Serialize, E: Error>(result: Result<T, E>) -> IronResult<Response> {
	let mut response = Response::new();
	match result {
		Ok(data) => {
			response.set_mut(JsonResponse::json(data))
					.set_mut(status::Ok);
			Ok(response)
		}
		Err(e) => {
			Err(bad_request(e.description()))
		}
	}
}

fn status(_: &mut Request) -> IronResult<Response> {
	Ok(Response::with((status::Ok, "running")))
}

fn weather(req: &mut Request) -> IronResult<Response> {
	let weather = req.get_database().get_latest_weather();
	json_response(weather)
}

fn temperature_history(req: &mut Request) -> IronResult<Response> {
	let time_period = get_time_period(req)?;
	json_response(req.get_database().get_temperature_history(time_period))
}

fn humidity_history(req: &mut Request) -> IronResult<Response> {
	let time_period = get_time_period(req)?;
	json_response(req.get_database().get_humidity_history(time_period))
}

fn pressure_history(req: &mut Request) -> IronResult<Response> {
	let time_period = get_time_period(req)?;
	json_response(req.get_database().get_pressure_history(time_period))
}

fn moisture_history(req: &mut Request) -> IronResult<Response> {
	let sensor: String = get_param(req, "sensor")?;
	let time_period = get_time_period(req)?;
	json_response(req.get_database().get_moisture_history(&sensor, time_period))
}

pub fn run(database: Database) {
	let mut router = Router::new();
	router.get("/status", status, "status");
	router.get("/weather", weather, "weather");
	router.get("/temperature/:start/:end", temperature_history, "temperature history");
	router.get("/humidity/:start/:end", humidity_history, "humidity history");
	router.get("/pressure/:start/:end", pressure_history, "pressure history");
	router.get("/moisture/:sensor/:start/:end", moisture_history, "moisture history");

	Iron::new(middleware::insert(router, database))
		.http("0.0.0.0:5000")
		.unwrap();
}
