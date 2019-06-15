use iron::prelude::*;
use iron::{status, typemap, BeforeMiddleware};
use iron::mime::*;
use router::Router;
use std::collections::HashMap;
use std::io::Result;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use super::error::internal_error;
use super::get_param;

const DEFAULT_WIDTH: u16 = 1280;
const DEFAULT_HEIGHT: u16 = 720;
const MAX_CACHE_SECONDS: u64 = 900;

type ImageCache = HashMap<(u16, u16), (SystemTime, Vec<u8>)>;

struct SharedImageCache {
	cache: Arc<Mutex<ImageCache>>
}

impl SharedImageCache {
	fn new() -> Self {
		SharedImageCache { cache: Arc::new(Mutex::new(HashMap::new())) }
	}
}

impl typemap::Key for SharedImageCache {
	type Value = Arc<Mutex<ImageCache>>;
}

impl BeforeMiddleware for SharedImageCache {
	fn before(&self, req: &mut Request) -> IronResult<()> {
		req.extensions.insert::<SharedImageCache>(self.cache.clone());
		Ok(())
	}
}

fn get_live_image(width: u16, height: u16) -> Result<Vec<u8>> {
	let output = Command::new("/opt/vc/bin/raspistill")
			.args(&["-o", "-", "-rot", "0", "-w", &format!("{}", width), "-h", &format!("{}", height)])
			.output()?;
	Ok(output.stdout)	
}

fn get_cached_image(width: u16, height: u16, cache: &Arc<Mutex<ImageCache>>) -> Result<Vec<u8>> {
	let mut cache = cache.lock().unwrap();
	let key = (width, height);

	match cache.get(&key) {
		Some((timestamp, image)) if !expired(timestamp) => Ok(image.clone()),
		_ => {
			let image = get_live_image(width, height);
			match image {
				Ok(ref image) => { cache.insert(key, (SystemTime::now(), image.clone())); }
				Err(ref e) => { warn!("unable to fetch image: {}", e) }
			}
			image
		}
	}
}

fn expired(timestamp: &SystemTime) -> bool {
	match timestamp.elapsed() {
		Ok(duration) if duration < Duration::from_secs(MAX_CACHE_SECONDS) => false,
		_ => true
	}
}

fn respond_with_image(image: Vec<u8>) -> IronResult<Response> {
	let content_type = Mime(
		TopLevel::Image,
		SubLevel::Jpeg,
		vec![]
	);

	let mut response = Response::new();
	response.set_mut(image)
		.set_mut(content_type)
		.set_mut(status::Ok);
	Ok(response)	
}

fn get_image(width: u16, height: u16, cache: &Arc<Mutex<ImageCache>>) -> IronResult<Response> {	
	match get_cached_image(width, height, cache) {
		Ok(image) => respond_with_image(image),
		Err(e) => Err(internal_error(&format!("Failed to capture image: {}", e)))
	}
}

fn capture_default(req: &mut Request) -> IronResult<Response> {
	let cache = req.extensions.get::<SharedImageCache>().unwrap();
	get_image(DEFAULT_WIDTH, DEFAULT_HEIGHT, cache)
}

fn capture_sized(req: &mut Request) -> IronResult<Response> {
	let width = get_param(req, "width")?;
	let height = get_param(req, "height")?;
	let cache = req.extensions.get::<SharedImageCache>().unwrap();
	get_image(width, height, cache)
}

pub fn api() -> Chain {
	let mut router = Router::new();
	router.get("/", capture_default, "default");
	router.get("/:width/:height", capture_sized, "sized");

	let mut chain = Chain::new(router);
	chain.link_before(SharedImageCache::new());
	chain
}