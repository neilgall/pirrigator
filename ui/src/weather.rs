use seed::prelude::*;
use crate::chart;
use crate::utils::*;
use common::weather::WeatherEvent;

#[derive(Clone, Debug)]
pub enum Model {
    NotLoaded,
    Loading,
    Loaded(Vec<WeatherEvent>),
    Failed(String)
}

impl Default for Model {
    fn default() -> Self { Model::NotLoaded }
}

#[derive(Clone)]
pub enum Message {
    Fetch(u32),
    Fetched(Vec<WeatherEvent>),
    Failed(JsValue)
}

fn chart(data: &Vec<WeatherEvent>, label: &str, y_min: Option<f64>, f: &dyn Fn(&WeatherEvent) -> f64) -> chart::Chart {
    chart::Chart {
        width: 600,
        height: 200,
        y_min,
        y_max: None,
        data: vec![
            chart::Series {
                label: label.to_string(),
                data: data.iter().map(|e| chart::DataPoint { time: e.system_time(), value: f(e) }).collect()
            }
        ],
        bars: vec![]
    }
}

pub fn render(model: &Model) -> Node<Message> {
    div![
        h2!["Weather"],
        button![simple_ev(Ev::Click, Message::Fetch(HOURS_6)), "Last 6 Hours"],
        button![simple_ev(Ev::Click, Message::Fetch(DAY)), "Last Day"],
        button![simple_ev(Ev::Click, Message::Fetch(DAYS_2)), "Last 2 Days"],
        button![simple_ev(Ev::Click, Message::Fetch(WEEK)), "Last Week"],
        match model {
            Model::NotLoaded =>
                p!["Select a time range"],
            Model::Loading =>
                p!["Loading..."],
            Model::Failed(e) =>
                p![e],
            Model::Loaded(data) =>
                div![
                    chart(data, "Temperature", Some(0.0), &|r| r.temperature).render().map_msg(|_| Message::Fetch(HOUR)),
                    chart(data, "Humidity", Some(0.0), &|r| r.humidity).render().map_msg(|_| Message::Fetch(HOUR)),
                    chart(data, "Barometric Pressure", None, &|r| r.pressure).render().map_msg(|_| Message::Fetch(HOUR))
                ]
        }
    ]
}

pub fn update(msg: Message, model: &mut Model, orders: &mut impl Orders<Message>) {
    match msg {
        Message::Fetch(t) => {
            let duration = t;
            orders.perform_cmd(async move {
                let request = Request::new(format!("/api/weather/-{}/-0", duration));
                let response = fetch(request).await.expect("fetch weather failed");
                let rows = response.check_status()
                                   .expect("statuc check failed")
                                   .json::<Vec<WeatherEvent>>()
                                   .await
                                   .expect("deserialisation failed");
                Message::Fetched(rows)
            });
            *model = Model::Loading;
        }
        Message::Fetched(rows) => {
            *model = if rows.is_empty() { Model::NotLoaded } else { Model::Loaded(rows) };
        }

        Message::Failed(e) => {
            *model = Model::Failed(e.as_string().unwrap_or("Unknown error".to_string()));
        }
    }
}

