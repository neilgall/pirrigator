use seed::prelude::*;
use crate::chart;
use crate::utils::*;
use common::weather::WeatherEvent;

type WeatherData = Vec<WeatherEvent>;

#[derive(Clone, Debug)]
pub enum Model {
    NotLoaded,
    Loading { duration: u32 },
    Loaded { duration: u32, data: WeatherData },
    Failed(String)
}

impl Default for Model {
    fn default() -> Self { Model::NotLoaded }
}

impl Model {
    fn selected_duration(&self) -> u32 {
        match self {
            Model::Loading { duration } => *duration,
            Model::Loaded { duration, data: _ } => *duration,
            _ => 0
        }
    }
}

#[derive(Clone)]
pub enum Message {
    Fetch { duration: u32 },
    Fetched { duration: u32, data: WeatherData },
    Failed(String)
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
    let selected = model.selected_duration();
    let buttons = vec![
        (HOURS_6, "Last 6 Hours"),
        (DAY, "Last Day"),
        (DAYS_2, "Last 2 Days"),
        (WEEK, "Last Week")
    ];
    div![
        h2!["Weather"],
        buttons.iter().map(|(duration, title)|
            button![
                attrs!{At::Class => if selected == *duration {SELECTED} else {UNSELECTED}},
                simple_ev(Ev::Click, Message::Fetch { duration: *duration }),
                title
            ]
        ),
        match model {
            Model::NotLoaded =>
                p![attrs!{At::Class => "placeholder"}, "Select a time range"],
            Model::Loading { duration: _ } =>
                p![attrs!{At::Class => "placeholder"}, "Loading..."],
            Model::Failed(e) =>
                p![attrs!{At::Class => "placeholder"}, e],
            Model::Loaded { duration: _, data } => {
                div![attrs!{At::Class => "chart"},
                    chart(data, "Temperature", Some(0.0), &|r| r.temperature)
                        .render()
                        .map_msg(|_| Message::Fetch { duration: DAY }),
                    chart(data, "Humidity", Some(0.0), &|r| r.humidity)
                        .render()
                        .map_msg(|_| Message::Fetch { duration: DAY }),
                    chart(data, "Barometric Pressure", None, &|r| r.pressure)
                        .render()
                        .map_msg(|_| Message::Fetch { duration: DAY })
                ]
            }
        }
    ]
}

pub fn update(msg: Message, model: &mut Model, orders: &mut impl Orders<Message>) {
    match msg {
        Message::Fetch { duration } => {
            orders.perform_cmd(fetch_weather(duration));
            *model = Model::Loading { duration };
        }
        Message::Fetched { duration, data } => {
            *model = if data.is_empty() { 
                Model::NotLoaded
            } else { 
                Model::Loaded { duration, data } 
            };
        }

        Message::Failed(e) => {
            *model = Model::Failed(e);
        }
    }
}

pub fn after_mount(orders: &mut impl Orders<Message>) {
    orders.send_msg(Message::Fetch { duration: DAY });
}

async fn fetch_weather(duration: u32) -> Message {
    let request = Request::new(format!("/api/weather/-{}/-0", duration));
    match fetch(request).await {
        Err(e) =>
            Message::Failed(format!("Failed to fetch weather: {:?}", e)),

        Ok(response) =>
            response.json::<Vec<WeatherEvent>>().await
                .map_or_else(
                    |e| Message::Failed(format!("Failed to parse weather data: {:?}", e)),
                    |data| Message::Fetched { duration, data }
                )
    }
}