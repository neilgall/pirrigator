use cached::SizedCache;
use iron::prelude::*;
use iron::status;
use iron::mime::*;
use router::Router;
use std::io::{Error, ErrorKind, Result};
use std::process::Command;
use std::time::{Duration, SystemTime};
use super::error::internal_error;
use super::get_param;

const DEFAULT_WIDTH: u16 = 1280;
const DEFAULT_HEIGHT: u16 = 720;
const MAX_CACHE_SECONDS: u64 = 900;

fn get_live_image(width: u16, height: u16) -> Result<Vec<u8>> {
	let output = Command::new("/opt/vc/bin/raspistill")
			.args(&["-o", "-", "-rot", "270", "-w", &format!("{}", width), "-h", &format!("{}", height)])
			.output()?;
	Ok(output.stdout)	
}

cached_control!{
	IMAGES: SizedCache<(u16, u16), (SystemTime, Vec<u8>)> = SizedCache::with_size(5);

	Key = { (width, height) };

	PostGet(cached) = {
		return SystemTime::now().duration_since(cached.0)
			.map_err(|_| Error::new(ErrorKind::Other, "time error"))
			.and_then(|duration| {
				if duration > Duration::from_secs(MAX_CACHE_SECONDS) {
					Err(Error::new(ErrorKind::NotFound, "expired"))
				} else {
					Ok(cached.1.clone())
				}
			}
		)
	};

	PostExec(result) = { result };

	Set(result) = { 
		let timestamp = SystemTime::now();
		match(result) {
			Ok(ref image) => (timestamp, image.clone()),
			Err(_) => (SystemTime::now(), vec![])
		}
	};

	Return(result) = { result };

	fn get_image(width: u16, height: u16) -> Result<Vec<u8>> = {
		get_live_image(width, height)
	}
}

fn capture(width: u16, height: u16) -> IronResult<Response> {
	let content_type = Mime(
		TopLevel::Image,
		SubLevel::Jpeg,
		vec![]
	);

	match get_image(width, height) {
		Ok(image) => {
			let mut response = Response::new();
			response.set_mut(image)
				.set_mut(content_type)
				.set_mut(status::Ok);
			Ok(response)
		}
		Err(e) => {
			Err(internal_error(&format!("Failed to capture image: {}", e)))
		}
	}
}

fn capture_default(_: &mut Request) -> IronResult<Response> {
	capture(DEFAULT_WIDTH, DEFAULT_HEIGHT)
}

fn capture_sized(req: &mut Request) -> IronResult<Response> {
	let width = get_param(req, "width")?;
	let height = get_param(req, "height")?;
	capture(width, height)
}

pub fn api() -> Router {
	let mut router = Router::new();
	router.get("/", capture_default, "default");
	router.get("/:width/:height", capture_sized, "sized");
	router
}