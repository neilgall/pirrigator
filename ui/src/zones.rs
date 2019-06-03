extern crate urlencoding;

use futures::Future;
use seed::prelude::*;
use seed::{Method, Request};
use std::time::{Duration, SystemTime};
use crate::utils::*;
use crate::chart;

#[derive(Clone, Debug, Deserialize)]
pub struct IrrigationRow {
    start: SystemTime,
    duration: Duration
}

#[derive(Clone, Debug, Deserialize)]
pub struct MoistureRow {
    timestamp: SystemTime,
    value: u16
}

#[derive(Clone, Debug, Deserialize)]
pub struct Zone {
    pub name: String,
    pub sensors: Vec<String>,
    pub irrigation: Vec<IrrigationRow>,
    pub moisture: HashMap<String, Vec<MoistureRow>>
}

#[derive(Clone, Debug)]
pub enum Model {
    NotLoaded,
    Loading,
    Loaded(Vec<Zone>),
    Failed(String)
}

impl Default for Model {
    fn default() -> Self { Model::NotLoaded }
}

#[derive(Clone)]
pub enum Message {
    FetchZones,
    FetchedZones(Vec<String>),
    FetchMoistureData(String, u32),
    FetchedMoistureData(String, Vec<(String, Vec<MoistureRow>)>, u32),
    FetchedIrrigationData(String, Vec<IrrigationRow>),
    Failed(JsValue)
}

impl From<&MoistureRow> for chart::DataPoint {
    fn from(m: &MoistureRow) -> Self {
        chart::DataPoint { time: m.timestamp.clone(), value: m.value as f64 }
    }
}

impl From<(&String, &Vec<MoistureRow>)> for chart::Series {
    fn from(s: (&String, &Vec<MoistureRow>)) -> Self {
        chart::Series { label: s.0.clone(), data: s.1.iter().map(chart::DataPoint::from).collect() }
    }
}

impl From<&IrrigationRow> for chart::Bar {
    fn from(r: &IrrigationRow) -> Self {
        chart::Bar { time: r.start, duration: r.duration }
    }
}

impl Zone {
    fn new(name: &str) -> Self {
        Zone {
            name: name.to_string(),
            sensors: vec![],
            irrigation: vec![],
            moisture: HashMap::new()
        }
    }

    fn fetch_moisture_data_event(&self, t: u32) -> Message {
        Message::FetchMoistureData(self.name.to_string(), t)
    }

    fn render(&self) -> El<Message> {
        div![
            h3![self.name],
            button![simple_ev(Ev::Click, self.fetch_moisture_data_event(HOURS_6)), "Last 6 Hours"],
            button![simple_ev(Ev::Click, self.fetch_moisture_data_event(DAY)), "Last Day"],
            button![simple_ev(Ev::Click, self.fetch_moisture_data_event(DAYS_2)), "Last 2 Days"],
            button![simple_ev(Ev::Click, self.fetch_moisture_data_event(WEEK)), "Last Week"],
            div![
                if self.moisture.is_empty() {
                    p!["Select a time range"]
                } else {
                    let c = chart::Chart {
                        width: 600,
                        height: 200,
                        y_min: None,
                        y_max: None,
                        data: self.moisture.iter().map(chart::Series::from).collect(),
                        bars: self.irrigation.iter().map(chart::Bar::from).collect()
                    };
                    c.render().map_message(|_| Message::FetchZones)
                }
            ]
        ]
    }
}

impl Model {
    pub fn render(&self) -> El<Message> {
        div![
            h2!["Zones"],
            match self {
                Model::NotLoaded => 
                    button![simple_ev(Ev::Click, Message::FetchZones), "Get Zones"],
                Model::Loading =>
                    p!["Loading..."],
                Model::Failed(e) =>
                    p![e],
                Model::Loaded(zones) => {
                    let els: Vec<El<Message>> = zones.iter().map(|z| z.render()).collect();
                    div![els]
                }
            }
        ]
    }

    fn zone(&mut self, name: &str) -> &mut Zone {
        if let Model::Loaded(ref mut zones) = self {
            zones.iter_mut().find(|ref z| z.name == name).unwrap()
        } else {
            panic!("unknown zone {}", name)
        }
    }

    pub fn update(&mut self, msg: Message) -> Update<Message> {
        match msg {
            Message::FetchZones => {
                *self = Model::Loading;
                Update::with_future_msg(self.fetch_zones()).render()
            }   
            Message::FetchedZones(zones) => {
                *self = Model::Loaded(zones.iter().map(|name| Zone::new(name)).collect());
                Render.into()
            }
            Message::FetchMoistureData(name, t) => {
                Update::with_future_msg(self.fetch_moisture_data(name, t)).render()
            }
            Message::FetchedMoistureData(name, moisture_data, t) => {
                let zone = self.zone(&name);
                for (sensor_name, data) in moisture_data {
                    zone.moisture.insert(sensor_name, data);
                }
                Update::with_future_msg(self.fetch_irrigation_data(name, t)).render()
            }
            Message::FetchedIrrigationData(name, irrigation_data) => {
                self.zone(&name).irrigation = irrigation_data;
                Render.into()
            }
            Message::Failed(e) => {
                *self = Model::Failed(e.as_string().unwrap_or("Unknown error".to_string()));
                Render.into()
            }
        }
    }

    fn fetch_zones(&self) -> impl Future<Item = Message, Error = Message> {
        Request::new("/api/zone/list")
            .method(Method::Get)
            .fetch_json()
            .map(Message::FetchedZones)
            .map_err(Message::Failed)
    }

    fn fetch_moisture_data(&mut self, name: String, duration: u32) -> impl Future<Item = Message, Error = Message> {
        let url = format!("/api/zone/{}/moisture/{}/0", urlencoding::encode(&name), duration);
        Request::new(&url)
            .method(Method::Get)
            .fetch_json()
            .map(move |data| Message::FetchedMoistureData(name, data, duration))
            .map_err(Message::Failed)
    }

    fn fetch_irrigation_data(&self, name: String, duration: u32) -> impl Future<Item = Message, Error = Message> {
        let url = format!("/api/zone/{}/irrigation/{}/0", urlencoding::encode(&name), duration);
        Request::new(&url)
            .method(Method::Get)
            .fetch_json()
            .map(|data| Message::FetchedIrrigationData(name, data))
            .map_err(Message::Failed)
    }
}
