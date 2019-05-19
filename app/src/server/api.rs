use crate::database::TimePeriod;

use iron::prelude::*;
use iron::status;
use router::Router;
use std::time::{Duration, SystemTime};
use super::get_param;
use super::json::json;
use super::middleware::DbRequestExtension;

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

fn moisture_sensors(req: &mut Request) -> IronResult<Response> {
	json(req.get_database().get_moisture_sensors())
}

fn moisture_history(req: &mut Request) -> IronResult<Response> {
	let sensor: String = get_param(req, "sensor")?;
	let time_period = get_time_period(req)?;
	json(req.get_database().get_moisture_history(&sensor, time_period))
}

fn irrigation_history(req: &mut Request) -> IronResult<Response> {
	let valve: String = get_param(req, "valve")?;
	let time_period = get_time_period(req)?;
	json(req.get_database().get_irrigation_history(&valve, time_period))
}

fn moisture_range_since_irrigation(req: &mut Request) -> IronResult<Response> {
	let sensor: String = get_param(req, "sensor")?;
	let valve: String = get_param(req, "valve")?;
	json(req.get_database().get_moisture_range_since_last_irrigation(&sensor, &valve))
}

pub fn api() -> Router {
	let mut router = Router::new();
	router.get("/status", status, "status");
	router.get("/weather", weather, "weather");
	router.get("/weather/:start/:end", weather_history, "weather history");
	router.get("/moisture/sensors", moisture_sensors, "moisture sensors");
	router.get("/moisture/:sensor/:start/:end", moisture_history, "moisture history");
	router.get("/moisture/range/:sensor/:valve", moisture_range_since_irrigation, "mean moisture");
	router.get("/irrigation/:valve/:start/:end", irrigation_history, "irrigation history");
	router	
}
