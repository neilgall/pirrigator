use iron::prelude::*;
use iron::status;
use router::Router;
use std::error::Error;
use std::time::Duration;

use crate::event::Event;
use crate::server::middleware::PirrigatorGet;
use common::time::{TimePeriod, UnixTime};

use super::error::*;
use super::get_param;
use super::json::*;

trait Timestamp {
	fn timestamp(&self) -> Result<UnixTime, Box<dyn Error>>;
}

impl Timestamp for String {
	fn timestamp(&self) -> Result<UnixTime, Box<dyn Error>> {
		if self.chars().next() == Some('-') {
			let delta = self[1..].parse()?;
			Ok(UnixTime::now() - Duration::from_secs(delta))
		} else {
			let timestamp = self.parse()?;
			Ok(UnixTime::from_timestamp(timestamp))
		}
	}
}

fn get_time_period(req: &Request) -> IronResult<TimePeriod> {
	let start = get_param::<String>(req, "start")?
				.timestamp()
				.map_err(|e| bad_request(&format!("cannot parse start time: {}", e.to_string())))?;

	let end = get_param::<String>(req, "end")?
				.timestamp()
				.map_err(|e| bad_request(&format!("cannot parse end time: {}", e.to_string())))?;

	Ok(TimePeriod { start, end })
}

fn status(_: &mut Request) -> IronResult<Response> {
	Ok(Response::with((status::Ok, "running")))
}

fn weather(req: &mut Request) -> IronResult<Response> {
	let weather = req.get_database()?.get_latest_weather();
	json_or_err(weather)
}

fn weather_history(req: &mut Request) -> IronResult<Response> {
	let time_period = get_time_period(req)?;
	json_or_err(req.get_database()?.get_weather_history(&time_period))
}

fn moisture_sensors(req: &mut Request) -> IronResult<Response> {
	json_or_err(req.get_database()?.get_moisture_sensors())
}

fn moisture_history(req: &mut Request) -> IronResult<Response> {
	let sensor: String = get_param(req, "sensor")?;
	let time_period = get_time_period(req)?;
	json_or_err(req.get_database()?.get_moisture_history(&sensor, &time_period))
}

fn irrigation_history(req: &mut Request) -> IronResult<Response> {
	let valve: String = get_param(req, "valve")?;
	let time_period = get_time_period(req)?;
	json_or_err(req.get_database()?.get_irrigation_history(&valve, &time_period))
}

fn moisture_range_since_irrigation(req: &mut Request) -> IronResult<Response> {
	let sensor: String = get_param(req, "sensor")?;
	let valve: String = get_param(req, "valve")?;
	json_or_err(req.get_database()?.get_moisture_range_since_last_irrigation(&sensor, &valve))
}

fn list_zones(req: &mut Request) -> IronResult<Response> {
	let names: Vec<String> = req.get_zones()?.keys().map(|z| z.to_string()).collect();
	json(&names)
}

fn moisture_history_for_zone(req: &mut Request) -> IronResult<Response> {
	let zone = req.get_zone()?;
	let time_period = get_time_period(req)?;
	let db = req.get_database()?;

	let data: Result<Vec<(String, Vec<(UnixTime, u16)>)>, rusqlite::Error> = zone.sensors.iter()
		.map(|sensor| Ok( (sensor.clone(), db.get_moisture_history(&sensor, &time_period)?) ))
		.collect();

	json_or_err(data)
}

fn irrigation_history_for_zone(req: &mut Request) -> IronResult<Response> {
	let zone = req.get_zone()?;
	let time_period = get_time_period(req)?;
	json_or_err(req.get_database()?.get_irrigation_history(&zone.valve, &time_period))
}

fn irrigate_zone(req: &mut Request) -> IronResult<Response> {
	let zone = req.get_zone()?;
	req.get_sender()?
		.send(Event::IrrigateEvent(zone.name))
		.map(|_| ok())
		.map_err(|_| internal_error("unable to send irrigate event"))
}

fn ok() -> Response {
	let mut response = Response::new();
	response.set_mut(status::Ok);
	response
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
	router.get("/zone/list", list_zones, "zones");
	router.get("/zone/:zone/moisture/:start/:end", moisture_history_for_zone, "moisture for zone");
	router.get("/zone/:zone/irrigation/:start/:end", irrigation_history_for_zone, "irrigation for zone");

	router.post("/zone/:zone/irrigate", irrigate_zone, "irrigate zone");

	router
}
