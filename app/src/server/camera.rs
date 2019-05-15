use cached::SizedCache;
use iron::prelude::*;
use iron::status;
use iron::mime::*;
use router::Router;
use std::process::Command;
use super::error::internal_error;
use super::get_param;

const DEFAULT_WIDTH: u16 = 1280;
const DEFAULT_HEIGHT: u16 = 720;

cached!{
	IMAGES: SizedCache<(u16, u16), Vec<u8>> = SizedCache::with_size(5);
	fn get_image(width: u16, height: u16) -> Vec<u8> = {
		let output = Command::new("/opt/vc/bin/raspistill")
			.args(&["-o", "-", "-w", &format!("{}", width), "-h", &format!("{}", height)])
			.output();

		match output {
			Ok(output) => {
				output.stdout
			}
			Err(e) => {
				error!("Failed to capture image: {}", e);
				vec![]
			}
		}
	}
}

fn capture(width: u16, height: u16) -> IronResult<Response> {
	let content_type = Mime(
		TopLevel::Image,
		SubLevel::Jpeg,
		vec![]
	);

	let image = get_image(width, height);
	if image.len() == 0 {
		Err(internal_error("Failed to capture image"))
	} else {
		let mut response = Response::new();
		response.set_mut(image)
				.set_mut(content_type)
				.set_mut(status::Ok);
		Ok(response)
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