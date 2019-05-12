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
    fn temperature_chart(&self) -> chart::Chart {
        chart::Chart {
            width: 600,
            height: 200,
            data: self.rows.iter().map(|r| chart::DataPoint { time: r.timestamp, value: r.temperature }).collect()
        }
    }

    pub fn render(&self) -> El<Message> {
        div![
            h2!["Weather"],
            button![simple_ev(Ev::Click, Message::Fetch(HOUR)), "Last Hour"],
            button![simple_ev(Ev::Click, Message::Fetch(DAY)), "Last Day"],
            button![simple_ev(Ev::Click, Message::Fetch(WEEK)), "Last Week"],
            button![simple_ev(Ev::Click, Message::Fetch(MONTH)), "Last Month"],
            if let Some(e) = &self.error {
                p![e]        
            } else {
                div![
                    self.temperature_chart().render().map_message(|_| Message::Fetch(HOUR))
                ]
            }
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
