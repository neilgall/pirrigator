use futures::Future;
use seed::prelude::*;
use seed::{Method, Request};
use std::iter::FromIterator;
use std::time::SystemTime;
use crate::utils::*;
use crate::chart;

#[derive(Clone, Debug, Deserialize)]
pub struct SensorRow {
    timestamp: SystemTime,
    value: u16
}

type SensorData = Vec<SensorRow>;

#[derive(Clone, Debug)]
pub enum Model {
    NotLoaded,
    Loaded(HashMap<String, SensorData>),
    Failed(String)
}

impl Default for Model {
    fn default() -> Self { Model::NotLoaded }
}

#[derive(Clone)]
pub enum Message {
    FetchNames,
    FetchedNames(Vec<String>),
    FetchData(String, u32),
    FetchedData(String, Vec<SensorRow>),
    Failed(JsValue)
}

fn render_sensor(name: &str, data: &SensorData) -> El<Message> {
    fn chart(data: &SensorData) -> chart::Chart {
        chart::Chart {
            width: 600,
            height: 200,
            y_origin_zero: true,
            data: data.iter().map(|SensorRow { timestamp: time, value }| chart::DataPoint { 
                time: time.clone(), 
                value: value.clone() as f64
            }).collect()
        }
    }
    div![
        h2![name],
        button![simple_ev(Ev::Click, Message::FetchData(name.to_string(), HOUR)), "Last Hour"],
        button![simple_ev(Ev::Click, Message::FetchData(name.to_string(), DAY)), "Last Day"],
        button![simple_ev(Ev::Click, Message::FetchData(name.to_string(), WEEK)), "Last Week"],
        button![simple_ev(Ev::Click, Message::FetchData(name.to_string(), MONTH)), "Last Month"],
        div![
            if data.is_empty() {
                p!["Select a time range"]
            } else {
                chart(&data).render().map_message(|_| Message::FetchNames)
            }
        ]
    ]
}

impl Model {
    pub fn render(&self) -> El<Message> {
        div![
            h2!["Sensors"],
            match self {
                Model::NotLoaded => 
                    button![simple_ev(Ev::Click, Message::FetchNames), "Get Sensors"],
                Model::Failed(e) =>
                    p![e],
                Model::Loaded(data) => {
                    let els: Vec<El<Message>> = data.iter().map(|(n, s)| render_sensor(n, s)).collect();
                    div![els]
                }
            }
        ]
    }

    pub fn update(&mut self, msg: Message) -> Update<Message> {
        match msg {
            Message::FetchNames => {
                *self = Model::NotLoaded;
                Update::with_future_msg(self.fetch_names()).skip()
            }
            Message::FetchedNames(names) => {
                let sensors = HashMap::from_iter(names.iter().map(|name| (name.to_string(), vec![])));
                *self = Model::Loaded(sensors);
                Render.into()
            }
            Message::FetchData(name, t) => {
                Update::with_future_msg(self.fetch_data(name, t)).skip()
            }
            Message::FetchedData(name, data) => {
                if let Model::Loaded(ref mut sensors) = self {
                    sensors.insert(name, data);
                }
                Render.into()
            }
            Message::Failed(e) => {
                *self = Model::Failed(e.as_string().unwrap_or("Unknown error".to_string()));
                Render.into()
            }
        }
    }

    pub fn fetch_names(&self) -> impl Future<Item = Message, Error = Message> {
        Request::new("/api/moisture/sensors")
            .method(Method::Get)
            .fetch_json()
            .map(Message::FetchedNames)
            .map_err(Message::Failed)
    }

    pub fn fetch_data(&mut self, name: String, duration: u32) -> impl Future<Item = Message, Error = Message> {
        let url = format!("/api/moisture/{}/{}/0", name, duration);
        Request::new(&url)
            .method(Method::Get)
            .fetch_json()
            .map(|data| Message::FetchedData(name, data))
            .map_err(Message::Failed)
    }
}
