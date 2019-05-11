use crate::database::TimePeriod;

use iron::prelude::*;
use iron::status;
use router::Router;
use std::str::FromStr;
use std::time::{Duration, SystemTime};
use super::error::bad_request;
use super::json::json;
use super::middleware::DbRequestExtension;

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

fn status(_: &mut Request) -> IronResult<Response> {
	Ok(Response::with((status::Ok, "running")))
}

fn weather(req: &mut Request) -> IronResult<Response> {
	let weather = req.get_database().get_latest_weather();
	json(weather)
}

fn weather_history(req: &mut Request) -> IronResult<Response> {
	let time_period = get_time_period(req)?;
	json(req.get_database().get_weather_history(time_period))
}

fn temperature_history(req: &mut Request) -> IronResult<Response> {
	let time_period = get_time_period(req)?;
	json(req.get_database().get_temperature_history(time_period))
}

fn humidity_history(req: &mut Request) -> IronResult<Response> {
	let time_period = get_time_period(req)?;
	json(req.get_database().get_humidity_history(time_period))
}

fn pressure_history(req: &mut Request) -> IronResult<Response> {
	let time_period = get_time_period(req)?;
	json(req.get_database().get_pressure_history(time_period))
}

fn moisture_sensors(req: &mut Request) -> IronResult<Response> {
	json(req.get_database().get_moisture_sensors())
}

fn moisture_history(req: &mut Request) -> IronResult<Response> {
	let sensor: String = get_param(req, "sensor")?;
	let time_period = get_time_period(req)?;
	json(req.get_database().get_moisture_history(&sensor, time_period))
}

pub fn api() -> Router {
	let mut router = Router::new();
	router.get("/status", status, "status");
	router.get("/weather", weather, "weather");
	router.get("/weather/:start/:end", weather_history, "weather history");
	router.get("/temperature/:start/:end", temperature_history, "temperature history");
	router.get("/humidity/:start/:end", humidity_history, "humidity history");
	router.get("/pressure/:start/:end", pressure_history, "pressure history");
	router.get("/moisture/sensors", moisture_sensors, "moisture sensors");
	router.get("/moisture/:sensor/:start/:end", moisture_history, "moisture history");
	router	
}