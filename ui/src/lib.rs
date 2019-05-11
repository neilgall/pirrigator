#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate seed;

use futures::Future;
use seed::prelude::*;
use seed::{Method, Request};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, Deserialize)]
struct Weather {
    timestamp: SystemTime,
    temperature: f64,
    humidity: f64,
    pressure: f64,
}

pub struct Pirrigator {
    weather: Vec<Weather>,
}

impl Default for Pirrigator {
    fn default() -> Self {
        Pirrigator { weather: vec![] }
    }
}

#[derive(Clone)]
enum Message {
    FetchWeather(u32),
    FetchedWeather(Vec<Weather>),
    FetchError(JsValue),
}

fn render_weather(model: &Pirrigator) -> El<Message> {
    fn weather_row(w: &Weather) -> El<Message> {
        tr![
            td![w.timestamp.duration_since(UNIX_EPOCH).unwrap().as_secs().to_string()],
            td![format!("{:.1}°C", w.temperature)],
            td![format!("{:.1}%", w.humidity)],
            td![format!("{:.0}mBar", w.pressure)]
        ]
    }

    let weather_items: Vec<El<Message>> = model.weather.iter().map(weather_row).collect();

    div![
        h2!["Weather"],
        button![simple_ev(Ev::Click, Message::FetchWeather(3600)), "Last Hour"],
        button![simple_ev(Ev::Click, Message::FetchWeather(86400)), "Last Day"],
        button![simple_ev(Ev::Click, Message::FetchWeather(86400*7)), "Last Week"],
        button![simple_ev(Ev::Click, Message::FetchWeather(86400*30)), "Last Month"],
        table![
            thead![
                tr![
                    th!["Time"],
                    th!["Temperature"],
                    th!["Humidity"],
                    th!["Pressure"]
                ],
            ],
            tbody![
                weather_items
            ]
        ]
    ]
}

fn get_weather(duration: u32) -> impl Future<Item = Message, Error = Message> {
    let url = format!("/api/weather/{}/0", duration);

    Request::new(&url)
        .method(Method::Get)
        .fetch_json()
        .map(Message::FetchedWeather)
        .map_err(Message::FetchError)
}

fn update(msg: Message, model: &mut Pirrigator) -> Update<Message> {
    match msg {
        Message::FetchWeather(t) => Update::with_future_msg(get_weather(t)).skip(),

        Message::FetchedWeather(w) => {
            model.weather = w;
            Render.into()
        }

        Message::FetchError(e) => {
            model.weather = vec![];
            Render.into()
        }
    }
}

fn view(model: &Pirrigator) -> El<Message> {
    div![
        h1!["Pirrigator"],
        render_weather(&model)
    ]
}

#[wasm_bindgen]
pub fn render() {
    seed::App::build(Pirrigator::default(), update, view)
        .finish()
        .run();
}
