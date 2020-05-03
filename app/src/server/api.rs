use iron::prelude::*;
use iron::{status, typemap, BeforeMiddleware};
use router::Router;
use std::collections::HashMap;
use std::error::Error;
use std::iter::FromIterator;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::controller::Zone;
use crate::time::{TimePeriod, UnixTime};

use super::error::bad_request;
use super::get_param;
use super::json::*;
use super::middleware::DbRequestExtension;

trait Timestamp {
	fn timestamp(&self) -> Result<SystemTime, Box<dyn Error>>;
}

impl Timestamp for String {
	fn timestamp(&self) -> Result<SystemTime, Box<dyn Error>> {
		if self.chars().next() == Some('-') {
			let delta = self[1..].parse()?;
			Ok(SystemTime::now() - Duration::from_secs(delta))
		} else {
			let timestamp = self.parse()?;
			Ok(UNIX_EPOCH + Duration::from_secs(timestamp))
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
	let weather = req.get_database().get_latest_weather();
	json_or_err(weather)
}

fn weather_history(req: &mut Request) -> IronResult<Response> {
	let time_period = get_time_period(req)?;
	json_or_err(req.get_database().get_weather_history(&time_period))
}

fn moisture_sensors(req: &mut Request) -> IronResult<Response> {
	json_or_err(req.get_database().get_moisture_sensors())
}

fn moisture_history(req: &mut Request) -> IronResult<Response> {
	let sensor: String = get_param(req, "sensor")?;
	let time_period = get_time_period(req)?;
	json_or_err(req.get_database().get_moisture_history(&sensor, &time_period))
}

fn irrigation_history(req: &mut Request) -> IronResult<Response> {
	let valve: String = get_param(req, "valve")?;
	let time_period = get_time_period(req)?;
	json_or_err(req.get_database().get_irrigation_history(&valve, &time_period))
}

fn moisture_range_since_irrigation(req: &mut Request) -> IronResult<Response> {
	let sensor: String = get_param(req, "sensor")?;
	let valve: String = get_param(req, "valve")?;
	json_or_err(req.get_database().get_moisture_range_since_last_irrigation(&sensor, &valve))
}

fn list_zones(req: &mut Request) -> IronResult<Response> {
	let zones = req.extensions.get::<ZonesMiddleware>().unwrap();
	let names: Vec<String> = zones.keys().map(|z| z.to_string()).collect();
	json(&names)
}

fn moisture_history_for_zone(req: &mut Request) -> IronResult<Response> {
	let zone = get_zone(req)?;
	let time_period = get_time_period(req)?;
	let db = req.get_database();

	let data: Result<Vec<(String, Vec<(UnixTime, u16)>)>, rusqlite::Error> = zone.sensors.iter()
		.map(|sensor| Ok( (sensor.clone(), db.get_moisture_history(&sensor, &time_period)?) ))
		.collect();

	json_or_err(data)
}

fn irrigation_history_for_zone(req: &mut Request) -> IronResult<Response> {
	let zone = get_zone(req)?;
	let time_period = get_time_period(req)?;
	json_or_err(req.get_database().get_irrigation_history(&zone.valve, &time_period))
}

struct ZonesMiddleware {
	zones: Arc<HashMap<String, Zone>>
}

impl typemap::Key for ZonesMiddleware {
	type Value = Arc<HashMap<String, Zone>>;
}

impl BeforeMiddleware for ZonesMiddleware {
	fn before(&self, req: &mut Request) -> IronResult<()> {
		req.extensions.insert::<ZonesMiddleware>(self.zones.clone());
		Ok(())
	}
}

fn get_zone(req: &mut Request) -> IronResult<Zone> {
	let name: String = get_param(req, "zone")?;
	let zones = req.extensions.get::<ZonesMiddleware>().unwrap();
	let zone = zones.get(&name).ok_or(bad_request(&format!("invalid zone {}", &name)))?;
	Ok(zone.clone())
}

pub fn api(zones: &Vec<Zone>) -> Chain {
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

	let zones = HashMap::from_iter(zones.iter().map(|z| (z.name.clone(), z.clone()) ));
	let mut chain = Chain::new(router);
	chain.link_before(ZonesMiddleware { zones: Arc::new(zones) });
	chain
}
