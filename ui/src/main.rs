#[macro_use] extern crate serde_derive;

use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};
use wasm_bindgen::prelude::*;

#[derive(Debug, Deserialize)]
struct Weather {
    timestamp: SystemTime,
    temperature: f64,
    humidity: f64,
    pressure: f64
}

#[derive(Debug)]
enum Fetch<T> {
    NotFetched,
    Fetching,
    Fetched(T),
    Failed(String)
}

impl<T> Default for Fetch<T> {
    fn default() -> Self {
        Fetch::NotFetched
    }
}

#[derive(Default)]
pub struct Pirrigator {
    weather: Fetch<Vec<Weather>>,
}

pub enum Message {
    FetchWeather(u32),
    FetchedWeather(Result<String, draco::fetch::Error>)
}

impl Pirrigator {
    fn render_weather(&self) -> draco::Node<Message> {
        use draco::html as h;
        let header = h::div()
            .push(h::h2().push("Weather"))
            .push(h::button().push("Last Hour").on("click", |_| Message::FetchWeather(3600)))
            .push(h::button().push("Last Day").on("click", |_| Message::FetchWeather(86400)))
            .push(h::button().push("Last Week").on("click", |_| Message::FetchWeather(86400*7)))
            .push(h::button().push("Last Month").on("click", |_| Message::FetchWeather(86400*30)));

        match &self.weather {
            Fetch::NotFetched =>
                header,
            Fetch::Fetching =>
                header.push("loading..."),
            Fetch::Fetched(weather) => {
                header.push(h::table()
                    .push(h::tr()
                        .push(h::th().push("Time"))
                        .push(h::th().push("Temperature"))
                        .push(h::th().push("Humidity"))
                        .push(h::th().push("Pressure")))
                    .append({
                        weather.iter().map(|w| h::tr()
                            .push(h::td().push(w.timestamp.duration_since(UNIX_EPOCH).unwrap().as_secs()))
                            .push(h::td().push(format!("{:.1}°C", w.temperature)))
                            .push(h::td().push(format!("{:.1}%", w.humidity)))
                            .push(h::td().push(format!("{:.0}mBar", w.pressure)))
                        )
                    })
                )
            },
            Fetch::Failed(e) =>
                header.push(format!("failed: {}", e))

        }.into()
    }
}

impl draco::App for Pirrigator {
    type Message = Message;

    fn update(&mut self, mailbox: &draco::Mailbox<Message>, message: Self::Message) {
        use self::Message::*;
        match message {
            FetchWeather(t) => {
                let url = format!("http://192.168.0.25:5000/api/weather/{}/0", t);
                mailbox.spawn(
                    draco::fetch::get(&url).send::<draco::fetch::Text>(),
                    Message::FetchedWeather
                );
                self.weather = Fetch::Fetching;
            }
            FetchedWeather(Ok(text)) => {
                match serde_json::from_str(&text) {
                    Ok(weather) => self.weather = Fetch::Fetched(weather),
                    Err(error) => self.weather = Fetch::Failed(error.description().to_string())
                }
    
            }
            FetchedWeather(Err(_)) => {
                self.weather = Fetch::Failed("Cannot fetch".to_string());
            }
        }
    }

    fn render(&self) -> draco::Node<Self::Message> {
        use draco::html as h;
        h::div()
            .push(h::h1().push("Pirrigator"))
            // .push(self.render_weather())
            .into()
    }
}

#[wasm_bindgen]
pub fn start() {
    draco::start(
        Pirrigator::default(),
        draco::select("main").expect("main").into(),
    );
}

pub fn main() {}
