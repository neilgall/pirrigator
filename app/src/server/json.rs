extern crate iron_json_response as ijr;

use iron::headers::ContentType;
use iron::prelude::*;
use iron::status;
use ijr::JsonResponse;
use serde::ser::Serialize;
use std::error::Error;
use super::error::bad_request;

pub fn json<T: Serialize>(data: &T) -> IronResult<Response> {
	let mut response = Response::new();
	response.set_mut(JsonResponse::json(data))
			.set_mut(status::Ok);
	response.headers.set(ContentType::json());
	Ok(response)	
}

pub fn json_or_err<T: Serialize, E: Error>(result: Result<T, E>) -> IronResult<Response> {
	match result  {
		Ok(ref data) => json(data),
		Err(e) => Err(bad_request(&e.to_string()))
	}
}
