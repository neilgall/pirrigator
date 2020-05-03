// #[macro_use] extern crate lazy_static;
// #[macro_use] extern crate lazy_static_include;
#[macro_use] extern crate log;
#[macro_use] extern crate serde_derive;

mod button;
mod database;
mod event;
mod moisture;
mod server;
mod time;
mod valve;
mod weather;

pub mod controller;
pub mod pirrigator;
pub mod settings;
