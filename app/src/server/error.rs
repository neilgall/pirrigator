use iron::prelude::*;
use iron::status;
use std::error::Error;

#[derive(Debug)]
struct ServerError {
	pub msg: String
}

impl ServerError {
	fn new(msg: &str) -> Self {
		ServerError { msg: msg.to_string() }
	}
}

impl std::fmt::Display for ServerError {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}", self.msg)
	}
}

impl Error for ServerError {
	fn description(&self) -> &str {
		&self.msg
	}
}

pub fn bad_request(msg: &str) -> IronError {
	IronError::new(ServerError::new(msg), status::BadRequest)
}

// pub fn internal_error(msg: &str) -> IronError {
// 	IronError::new(ServerError::new(msg), status::InternalServerError)
// }
