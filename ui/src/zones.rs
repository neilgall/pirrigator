extern crate urlencoding;

use seed::prelude::*;
use std::collections::HashMap;
use std::time::Duration;
use crate::utils::*;
use crate::chart;
use common::moisture::Measurement;
use common::time::{TimeSeries, UnixTime};

#[derive(Clone, Debug, Deserialize)]
pub struct IrrigationRow {
    start: UnixTime,
    duration: Duration
}

#[derive(Clone, Debug, Deserialize)]
pub struct Zone {
    pub name: String,
    pub sensors: Vec<String>,
    pub irrigation: Vec<IrrigationRow>,
    pub moisture: HashMap<String, TimeSeries<Measurement>>
}

#[derive(Clone, Debug)]
pub enum Model {
    NotLoaded,
    Loading,
    Loaded(Vec<Zone>),
    Failed(String)
}

type MoistureRow = (UnixTime, Measurement);
type MoistureData = Vec<(String, TimeSeries<Measurement>)>;
type IrrigationData = Vec<IrrigationRow>;

impl Default for Model {
    fn default() -> Self { Model::NotLoaded }
}

#[derive(Clone)]
pub enum Message {
    FetchZones,
    FetchedZones(Vec<String>),
    FetchMoistureData(String, u32),
    FetchedMoistureData(String, MoistureData, u32),
    FetchedIrrigationData(String, IrrigationData),
    Failed(JsValue)
}

impl From<&MoistureRow> for chart::DataPoint {
    fn from(m: &MoistureRow) -> Self {
        chart::DataPoint { time: m.0.system_time(), value: m.1 as f64 }
    }
}

impl From<(&String, &Vec<MoistureRow>)> for chart::Series {
    fn from(s: (&String, &Vec<MoistureRow>)) -> Self {
        chart::Series { label: s.0.clone(), data: s.1.iter().map(chart::DataPoint::from).collect() }
    }
}

impl From<&IrrigationRow> for chart::Bar {
    fn from(r: &IrrigationRow) -> Self {
        chart::Bar { time: r.start.system_time(), duration: r.duration }
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

    fn render(&self) -> Node<Message> {
        div![
            h3![&self.name],
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
                    c.render().map_msg(|_| Message::FetchZones)
                }
            ]
        ]
    }
}

impl Model {
    fn zone(&mut self, name: &str) -> &mut Zone {
        if let Model::Loaded(ref mut zones) = self {
            zones.iter_mut().find(|ref z| z.name == name).unwrap()
        } else {
            panic!("unknown zone {}", name)
        }
    }
}

pub fn render(model: &Model) -> Node<Message> {
    div![
        h2!["Zones"],
        match model {
            Model::NotLoaded => 
                button![simple_ev(Ev::Click, Message::FetchZones), "Get Zones"],
            Model::Loading =>
                p!["Loading..."],
            Model::Failed(e) =>
                p![e],
            Model::Loaded(zones) => {
                let els: Vec<Node<Message>> = zones.iter().map(|z| z.render()).collect();
                div![els]
            }
        }
    ]
}

pub fn update(msg: Message, model: &mut Model, orders: &mut impl Orders<Message>) {
    match msg {
        Message::FetchZones => {
            orders.perform_cmd(async {
                let response = fetch("/api/zone/list").await.expect("fetch zones failed");
                let zones = response.check_status()
                                    .expect("status check failed")
                                    .json::<Vec<String>>()
                                    .await
                                    .expect("deserialisation failed");
                Message::FetchedZones(zones)
            });
            *model = Model::Loading;
        }   
        Message::FetchedZones(zones) => {
            *model = Model::Loaded(zones.iter().map(|name| Zone::new(name)).collect());
        }
        Message::FetchMoistureData(name, t) => {
            let duration = t;
            orders.perform_cmd(async move {
                let request = Request::new(format!("/api/zone/{}/moisture/-{}/-0", urlencoding::encode(&name), duration));
                let response = fetch(request).await.expect("failed to fetch zone data");
                let data = response.check_status()
                                   .expect("status check failed")
                                   .json::<MoistureData>()
                                   .await
                                   .expect("deserialisation failed");
                Message::FetchedMoistureData(name, data, duration)
            });
        }
        Message::FetchedMoistureData(name, moisture_data, t) => {
            let zone = model.zone(&name);
            for (sensor_name, data) in moisture_data {
                zone.moisture.insert(sensor_name, data);
            }

            let duration = t;
            orders.perform_cmd(async move {
                let request = Request::new(format!("/api/zone/{}/irrigation/-{}/-0", urlencoding::encode(&name), duration));
                let response = fetch(request).await.expect("failed to fetch irrigation data");
                let data = response.check_status()
                                   .expect("status check failed")
                                   .json::<IrrigationData>()
                                   .await
                                   .expect("deserialisation failed");
                Message::FetchedIrrigationData(name, data)
            });
        }
        Message::FetchedIrrigationData(name, irrigation_data) => {
            model.zone(&name).irrigation = irrigation_data;
        }
        Message::Failed(e) => {
            *model = Model::Failed(e.as_string().unwrap_or("Unknown error".to_string()));
        }
    }
}
