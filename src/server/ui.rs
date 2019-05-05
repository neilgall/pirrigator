use iron::prelude::*;
use iron::status;
use iron::mime::*;
use router::Router;

lazy_static_include_bytes!(INDEX, "../draco/target/examples/svg_clock/index.html");
lazy_static_include_bytes!(START_JS, "../draco/target/examples/svg_clock/svg_clock.js");
lazy_static_include_bytes!(WASM, "../draco/target/examples/svg_clock/svg_clock_bg.wasm");

fn respond(data: &[u8], content_type: Mime) -> IronResult<Response> {
	let mut response = Response::new();
	response.set_mut(data)
			.set_mut(content_type)
			.set_mut(status::Ok);
	Ok(response)
}

fn index_html(_: &mut Request) -> IronResult<Response> {
	let content_type = Mime(
		TopLevel::Text,
		SubLevel::Html,
		vec![(iron::mime::Attr::Charset, iron::mime::Value::Utf8)]
	);
	respond(*INDEX, content_type)
}

fn draco_starter_js(_: &mut Request) -> IronResult<Response> {
	let content_type = Mime(
		TopLevel::Application,
		SubLevel::Javascript,
		vec![(iron::mime::Attr::Charset, iron::mime::Value::Utf8)]
	);
	respond(*START_JS, content_type)
}

fn draco_starter_wasm(_: &mut Request) -> IronResult<Response> {
	let content_type = Mime(
		TopLevel::Application,
		SubLevel::Ext(String::from("wasm")),
		vec![]
	);
	respond(*WASM, content_type)
}

pub fn ui() -> Router {
	let mut router = Router::new();
	router.get("/", index_html, "root");
	router.get("/index.html", index_html, "index.html");
	router.get("/svg_clock.js", draco_starter_js, "start js");
	router.get("/svg_clock_bg.wasm", draco_starter_wasm, "wasm blob");
	router			
}