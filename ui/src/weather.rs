use futures::Future;
use seed::prelude::*;
use seed::{Method, Request};
use std::time::SystemTime;
use crate::utils::render_system_time;

const HOUR: u32 = 3600;
const DAY: u32 = 86400;
const WEEK: u32 = 86400*7;
const MONTH: u32 = 86400*30;

#[derive(Clone, Debug, Deserialize)]
pub struct WeatherRow {
    timestamp: SystemTime,
    temperature: f64,
    humidity: f64,
    pressure: f64,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Weather {
    rows: Vec<WeatherRow>,
    error: Option<String>
}

#[derive(Clone)]
pub enum Message {
    Fetch(u32),
    Fetched(Vec<WeatherRow>),
    Failed(JsValue)
}

impl Weather {
    pub fn render(&self) -> El<Message> {
        fn row(w: &WeatherRow) -> El<Message> {
            tr![
                td![render_system_time(w.timestamp)],
                td![format!("{:.1}°C", w.temperature)],
                td![format!("{:.1}%", w.humidity)],
                td![format!("{:.0}mBar", w.pressure)]
            ]
        }

        let weather_items: Vec<El<Message>> = self.rows.iter().map(row).collect();

        div![
            h2!["Weather"],
            button![simple_ev(Ev::Click, Message::Fetch(HOUR)), "Last Hour"],
            button![simple_ev(Ev::Click, Message::Fetch(DAY)), "Last Day"],
            button![simple_ev(Ev::Click, Message::Fetch(WEEK)), "Last Week"],
            button![simple_ev(Ev::Click, Message::Fetch(MONTH)), "Last Month"],
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

    pub fn update(&mut self, msg: Message) -> Update<Message> {
        match msg {
            Message::Fetch(t) => {
                self.rows = vec![];
                self.error = None;
                Update::with_future_msg(self.fetch(t)).skip()
            }
            Message::Fetched(rows) => {
                self.rows = rows;
                self.error = None;
                Render.into()
            }

            Message::Failed(e) => {
                self.rows = vec![];
                self.error = e.as_string();
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
