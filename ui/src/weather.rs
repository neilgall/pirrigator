use futures::Future;
use seed::prelude::*;
use seed::{Method, Request};
use std::time::SystemTime;
use crate::chart;
use crate::utils::*;

#[derive(Clone, Debug, Deserialize)]
pub struct WeatherRow {
    timestamp: SystemTime,
    temperature: f64,
    humidity: f64,
    pressure: f64,
}

#[derive(Clone, Debug)]
pub enum Model {
    NotLoaded,
    Loading,
    Loaded(Vec<WeatherRow>),
    Failed(String)
}

impl Default for Model {
    fn default() -> Self { Model::NotLoaded }
}

#[derive(Clone)]
pub enum Message {
    Fetch(u32),
    Fetched(Vec<WeatherRow>),
    Failed(JsValue)
}

fn chart(data: &Vec<WeatherRow>, label: &str, y_min: Option<f64>, f: &Fn(&WeatherRow) -> f64) -> chart::Chart {
    chart::Chart {
        width: 600,
        height: 200,
        y_min,
        y_max: None,
        data: vec![
            chart::Series {
                label: label.to_string(),
                data: data.iter().map(|r| chart::DataPoint { time: r.timestamp, value: f(r) }).collect()
            }
        ],
        bars: vec![]
    }
}

impl Model {
    pub fn render(&self) -> El<Message> {
        div![
            h2!["Weather"],
            button![simple_ev(Ev::Click, Message::Fetch(HOUR)), "Last Hour"],
            button![simple_ev(Ev::Click, Message::Fetch(DAY)), "Last Day"],
            button![simple_ev(Ev::Click, Message::Fetch(WEEK)), "Last Week"],
            button![simple_ev(Ev::Click, Message::Fetch(MONTH)), "Last Month"],
            match self {
                Model::NotLoaded =>
                    p!["Select a time range"],
                Model::Loading =>
                    p!["Loading..."],
                Model::Failed(e) =>
                    p![e],
                Model::Loaded(data) =>
                    div![
                        chart(data, "Temperature", Some(0.0), &|r| r.temperature).render().map_message(|_| Message::Fetch(HOUR)),
                        chart(data, "Humidity", Some(0.0), &|r| r.humidity).render().map_message(|_| Message::Fetch(HOUR)),
                        chart(data, "Barometric Pressure", None, &|r| r.pressure).render().map_message(|_| Message::Fetch(HOUR))
                    ]
            }
        ]
    }

    pub fn update(&mut self, msg: Message) -> Update<Message> {
        match msg {
            Message::Fetch(t) => {
                *self = Model::Loading;
                Update::with_future_msg(self.fetch(t)).render()
            }
            Message::Fetched(rows) => {
                *self = if rows.is_empty() { Model::NotLoaded } else { Model::Loaded(rows) };
                Render.into()
            }

            Message::Failed(e) => {
                *self = Model::Failed(e.as_string().unwrap_or("Unknown error".to_string()));
                Render.into()
            }
        }
    }

    pub fn fetch(&self, duration: u32) -> impl Future<Item = Message, Error = Message> {
        let url = format!("/api/weather/{}/0", duration);

        Request::new(&url)
            .method(Method::Get)
            .fetch_json()
            .map(Message::Fetched)
            .map_err(Message::Failed)
    }
}
