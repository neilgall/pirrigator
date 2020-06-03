use iron::prelude::*;
use iron::status;
use iron::mime::*;
use router::Router;

lazy_static_include_bytes!(INDEX, "../ui/index.html");
lazy_static_include_bytes!(CSS, "../ui/pirrigator.css");
lazy_static_include_bytes!(START_JS, "../ui/target/html/release/pirrigator-ui.js");
lazy_static_include_bytes!(WASM, "../ui/target/html/release/pirrigator-ui_bg.wasm");

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

fn css(_: &mut Request) -> IronResult<Response> {
	let content_type = Mime(
		TopLevel::Text,
		SubLevel::Css,
		vec![(iron::mime::Attr::Charset, iron::mime::Value::Utf8)]
	);
	respond(*CSS, content_type)
}

fn javascript(_: &mut Request) -> IronResult<Response> {
	let content_type = Mime(
		TopLevel::Application,
		SubLevel::Javascript,
		vec![(iron::mime::Attr::Charset, iron::mime::Value::Utf8)]
	);
	respond(*START_JS, content_type)
}

fn wasm(_: &mut Request) -> IronResult<Response> {
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
	router.get("/pirrigator.css", css, "css");
	router.get("/pirrigator-ui.js", javascript, "start js");
	router.get("/pirrigator-ui_bg.wasm", wasm, "wasm blob");
	router			
}