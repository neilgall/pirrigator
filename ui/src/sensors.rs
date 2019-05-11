use futures::Future;
use seed::prelude::*;
use seed::{Method, Request};
use std::iter::FromIterator;
use std::time::SystemTime;
use crate::utils::*;

#[derive(Clone, Debug, Deserialize)]
pub struct SensorRow {
    timestamp: SystemTime,
    value: u16
}

type SensorData = Vec<SensorRow>;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Sensors {
    sensors: HashMap<String, SensorData>,
    error: Option<String>
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
    fn row(row: &SensorRow) -> El<Message> {
        tr![
            td![render_system_time(row.timestamp)],
            td![format!("{}", row.value)]
        ]
    }
    let rows: Vec<El<Message>> = data.iter().map(row).collect();

    div![
        h2![name],
        button![simple_ev(Ev::Click, Message::FetchData(name.to_string(), HOUR)), "Last Hour"],
        button![simple_ev(Ev::Click, Message::FetchData(name.to_string(), DAY)), "Last Day"],
        button![simple_ev(Ev::Click, Message::FetchData(name.to_string(), WEEK)), "Last Week"],
        button![simple_ev(Ev::Click, Message::FetchData(name.to_string(), MONTH)), "Last Month"],
        table![
            thead![
                tr![
                    th!["Time"],
                    th!["Reading"],
                ],
            ],
            tbody![rows]
        ]
    ]

}

impl Sensors {
    pub fn render(&self) -> El<Message> {
        let sensors: Vec<El<Message>> = self.sensors.iter().map(|(n, s)| render_sensor(n, s)).collect();
        div![
            h2!["Sensors"],
            if let Some(e) = &self.error {
                p![e]
            } else if self.sensors.is_empty() {
                button![simple_ev(Ev::Click, Message::FetchNames), "Get Sensors"]
            } else {
                div![sensors]
            }
        ]
    }

    pub fn update(&mut self, msg: Message) -> Update<Message> {
        match msg {
            Message::FetchNames => {
                self.sensors = HashMap::new();
                self.error = None;
                Update::with_future_msg(self.fetch_names()).skip()
            }
            Message::FetchedNames(names) => {
                self.sensors = HashMap::from_iter(names.iter().map(|name| (name.to_string(), vec![])));
                self.error = None;
                Render.into()
            }
            Message::FetchData(name, t) => {
                Update::with_future_msg(self.fetch_data(name, t)).skip()
            }
            Message::FetchedData(name, data) => {
                self.sensors.insert(name, data);
                Render.into()
            }
            Message::Failed(e) => {
                self.sensors = HashMap::new();
                self.error = e.as_string();
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
