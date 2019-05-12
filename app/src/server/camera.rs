use iron::prelude::*;
use iron::status;
use iron::mime::*;
use router::Router;
use std::process::Command;
use super::get_param;
use super::error::internal_error;

const DEFAULT_WIDTH: u16 = 1280;
const DEFAULT_HEIGHT: u16 = 720;

fn capture(width: u16, height: u16, exposure: &str) -> IronResult<Response> {
	let output = Command::new("/opt/vc/bin/raspistill")
		.args(&["-o", "-", "-w", &format!("{}", width), "-h", &format!("{}", height), "-ex", exposure])
		.output()
		.map_err(|_| internal_error("Failed to capture image"))?;

	let content_type = Mime(
		TopLevel::Image,
		SubLevel::Jpeg,
		vec![]
	);

	let mut response = Response::new();
	response.set_mut(output.stdout)
			.set_mut(content_type)
			.set_mut(status::Ok);
	Ok(response)
}

fn capture_default(_: &mut Request) -> IronResult<Response> {
	capture(DEFAULT_WIDTH, DEFAULT_HEIGHT, "auto")
}

fn capture_night(_: &mut Request) -> IronResult<Response> {
	capture(DEFAULT_WIDTH, DEFAULT_HEIGHT, "night")
}

fn capture_sized(req: &mut Request) -> IronResult<Response> {
	let width = get_param(req, "width")?;
	let height = get_param(req, "height")?;
	capture(width, height, "auto")
}

pub fn api() -> Router {
	let mut router = Router::new();
	router.get("/", capture_default, "default");
	router.get("/night", capture_night, "night mode");
	router.get("/:width/:height", capture_sized, "sized");
	router
}