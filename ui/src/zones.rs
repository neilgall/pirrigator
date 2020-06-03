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

type ZoneMoistureData = HashMap<String, TimeSeries<Measurement>>;
type ZoneIrrigationData = Vec<IrrigationRow>;

#[derive(Clone, Debug)]
pub enum ZoneData {
    NotLoaded,
    Loading,
    LoadedMoisture { duration: u32, moisture: ZoneMoistureData },
    LoadedAll { duration: u32, moisture: ZoneMoistureData, irrigation: ZoneIrrigationData }
}

#[derive(Clone, Debug)]
pub struct Zone {
    pub name: String,
    pub sensors: Vec<String>,
    pub data: ZoneData
}

#[derive(Clone, Debug)]
pub enum Model {
    NotLoaded,
    Loading,
    Loaded { zones: Vec<Zone> },
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
    FetchMoistureData { zone: String, duration: u32 },
    FetchedMoistureData { zone: String, data: MoistureData, duration: u32 },
    FetchedIrrigationData { zone: String, data: IrrigationData },
    Failed(String)
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
            data: ZoneData::NotLoaded
        }
    }

    fn duration(&self) -> u32 {
        match self.data {
            ZoneData::LoadedMoisture { duration, moisture: _ } => duration,
            ZoneData::LoadedAll { duration, moisture: _, irrigation: _ } => duration,
            _ => 0
        }
    }

    fn render(&self) -> Node<Message> {
        let buttons = vec![
            (HOURS_6, "Last 6 Hours"),
            (DAY, "Last Day"),
            (DAYS_2, "Last 2 Days"),
            (WEEK, "Last Week")
        ];

        div![
            attrs![At::Class => "zone"],
            
            h3![&self.name],

            buttons.iter().map(|(duration, title)|
                button![
                    title,
                    attrs!{At::Class => if *duration == self.duration() {SELECTED} else {UNSELECTED}},
                    simple_ev(Ev::Click, Message::FetchMoistureData { zone: self.name.clone(), duration: *duration })
                ]
            ),

            div![
                match self.data {
                    ZoneData::NotLoaded => 
                        p![attrs!{At::Class => "placeholder"}, "Select a time range"],

                    ZoneData::Loading =>
                        p![attrs!{At::Class => "placeholder"}, "Fetching data..."],

                    ZoneData::LoadedMoisture { duration: _, ref moisture } => 
                        div![attrs!{At::Class => "chart"}, self.render_chart(&moisture, &vec![])],

                    ZoneData::LoadedAll { duration: _, ref moisture, ref irrigation } =>
                        div![attrs!{At::Class => "chart"}, self.render_chart(&moisture, &irrigation)]
                }
            ]
        ]
    }

    fn render_chart(&self, moisture: &ZoneMoistureData, irrigation: &ZoneIrrigationData) -> Node<Message> {
       let c = chart::Chart {
            width: 600,
            height: 200,
            y_min: None,
            y_max: None,
            data: moisture.iter().map(chart::Series::from).collect(),
            bars: irrigation.iter().map(chart::Bar::from).collect()
        };
        c.render().map_msg(|_| Message::FetchZones)
    }
}

impl Model {
    fn zone(&mut self, name: &str) -> &mut Zone {
        if let Model::Loaded { ref mut zones } = self {
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
                button![
                    attrs!{At::Class => "placeholder"},
                    simple_ev(Ev::Click, Message::FetchZones),
                    "Get Zones"
                ],
            Model::Loading =>
                div![
                    attrs!{At::Class => "placeholder"},
                    p!["Fetching..."]
                ],
            Model::Failed(e) =>
                div![
                    attrs!{At::Class => "placeholder"}, 
                    p![e],
                    button![simple_ev(Ev::Click, Message::FetchZones), "Try Again"]
                ],
            Model::Loaded { zones } => {
                let els: Vec<Node<Message>> = zones.iter().map(|z| z.render()).collect();
                div![els]
            }
        }
    ]
}

pub fn update(msg: Message, model: &mut Model, orders: &mut impl Orders<Message>) {
    match msg {
        Message::FetchZones => {
            orders.perform_cmd(fetch_zones());
            *model = Model::Loading;
        }   
        Message::FetchedZones(zones) => {
            *model = Model::Loaded { zones: zones.iter().map(|name| Zone::new(name)).collect() };
        }
        Message::FetchMoistureData { ref zone, duration } => {
            orders.perform_cmd(fetch_moisture_data(zone.clone(), duration));
            model.zone(&zone).data = ZoneData::Loading;
        }
        Message::FetchedMoistureData { zone: name, data, duration } => {
            let zone = model.zone(&name);
            let mut moisture = HashMap::new();
            for (sensor_name, data) in data {
                moisture.insert(sensor_name, data);
            }
            zone.data = ZoneData::LoadedMoisture { duration, moisture };

            orders.perform_cmd(fetch_irrigation_data(name, duration));
        }
        Message::FetchedIrrigationData { zone: name, data } => {
            let zone = model.zone(&name);
            if let ZoneData::LoadedMoisture { duration, ref moisture } = zone.data {
                zone.data = ZoneData::LoadedAll { duration, moisture: moisture.clone(), irrigation: data }
            }
        }
        Message::Failed(e) => {
            *model = Model::Failed(e);
        }
    }
}

pub fn after_mount(orders: &mut impl Orders<Message>) {
    orders.send_msg(Message::FetchZones);
}

async fn fetch_zones() -> Message {
    let request = Request::new("/api/zone/list");
    match fetch(request).await {
        Err(e) =>
            Message::Failed(format!("Failed to fetch zone list: {:?}", e)),

        Ok(response) =>
            response.json::<Vec<String>>().await.map_or_else(
                |e| Message::Failed(format!("Failed to parse zone list: {:?}", e)),
                Message::FetchedZones
            )
    }
}

async fn fetch_moisture_data(name: String, duration: u32) -> Message {
    let request = Request::new(format!("/api/zone/{}/moisture/-{}/-0", urlencoding::encode(&name), duration));
    match fetch(request).await {
        Err(e) => 
            Message::Failed(format!("Failed to fetch zone data: {:?}", e)),

        Ok(response) =>
            response.json::<MoistureData>().await.map_or_else(
                |e| Message::Failed(format!("Failed to parse zone data: {:?}", e)),
                |data| Message::FetchedMoistureData { zone: name, data, duration }
            )
    }
}

async fn fetch_irrigation_data(name: String, duration: u32) -> Message {
    let request = Request::new(format!("/api/zone/{}/irrigation/-{}/-0", urlencoding::encode(&name), duration));
    match fetch(request).await {
        Err(e) =>
            Message::Failed(format!("Failed to fetch irrigation data {:?}", e)),

        Ok(response) =>
            response.json::<IrrigationData>().await.map_or_else(
                |e| Message::Failed(format!("Failed to parse irrigation data: {:?}", e)),
                |data| Message::FetchedIrrigationData { zone: name, data }
            )
    }
}
