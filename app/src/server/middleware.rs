use crate::controller::Zone;
use crate::database::Database;
use crate::event::Event;
use crate::server::get_param;
use crate::server::error::*;

use iron::typemap::Key;
use iron::middleware::Handler;
use iron::prelude::*;
use persistent::Read;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::sync::{Arc, Mutex, mpsc::Sender};

pub struct PirrigatorData {
	database: Database,
	zones: HashMap<String, Zone>,
	tx: Arc<Mutex<Sender<Event>>>
}

impl PirrigatorData {
	pub fn new(database: Database, zones: &Vec<Zone>, tx: Sender<Event>) -> PirrigatorData {
		PirrigatorData {
			database,
			tx: Arc::new(Mutex::new(tx)),
			zones: HashMap::from_iter(zones.iter().map(|z| (z.name.clone(), z.clone()) ))
		}
	}
}

struct PirrigatorMiddleware;
impl Key for PirrigatorMiddleware { type Value = PirrigatorData; }

pub trait PirrigatorGet {
	fn get_database(&mut self) -> IronResult<Database>;
	fn get_zones(&mut self) -> IronResult<HashMap<String, Zone>>;
	fn get_zone(&mut self) -> IronResult<Zone>;
	fn get_sender(&mut self) -> IronResult<Sender<Event>>;
}

impl<'a, 'b> PirrigatorGet for Request<'a, 'b> {
	fn get_database(&mut self) -> IronResult<Database> {
		self.get::<Read<PirrigatorMiddleware>>()
			.map(|data| data.database.clone())
			.map_err(|_| internal_error("cannot get database connection"))
	}

	fn get_zones(&mut self) -> IronResult<HashMap<String, Zone>> {
		self.get::<Read<PirrigatorMiddleware>>()
			.map(|data| data.zones.clone())
			.map_err(|_| internal_error("cannot get zone information"))
	}

	fn get_zone(&mut self) -> IronResult<Zone> {
		let name: String = get_param(self, "zone")?;
		let zones = self.get_zones()?;
		let zone = zones.get(&name)
			.ok_or(bad_request(&format!("invalid zone {}", &name)))?;
		Ok(zone.clone())
	}

	fn get_sender(&mut self) -> IronResult<Sender<Event>> {
		self.get::<Read<PirrigatorMiddleware>>()
			.map(|data| data.tx.lock().unwrap().clone())
			.map_err(|_| internal_error("cannot get mpsc sender"))
	}
}

pub fn insert<H: Handler>(handler: H, pirrigator_data: PirrigatorData) -> impl Handler {
	let (logger_before, logger_after) = logger::Logger::new(None);
	let mut chain = Chain::new(handler);
	chain.link_before(logger_before);
	chain.link_before(Read::<PirrigatorMiddleware>::one(pirrigator_data));
	chain.link_after(iron_json_response::JsonResponseMiddleware::new());
	chain.link_after(logger_after);
	chain	
}
