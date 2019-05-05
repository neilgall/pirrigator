use iron::prelude::*;
use iron::status;
use std::error::Error;

#[derive(Debug)]
pub struct RequestError {
	pub msg: String
}

impl RequestError {
	pub fn new(msg: &str) -> Self {
		RequestError { msg: msg.to_string() }
	}
}

impl std::fmt::Display for RequestError {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}", self.msg)
	}
}

impl Error for RequestError {
	fn description(&self) -> &str {
		&self.msg
	}
}

pub fn bad_request(msg: &str) -> IronError {
	IronError::new(RequestError::new(msg), status::BadRequest)
}
