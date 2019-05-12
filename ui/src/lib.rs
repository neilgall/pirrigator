#[macro_use] extern crate serde_derive;
#[macro_use] extern crate seed;

mod chart;
mod sensors;
mod weather;
mod utils;

use seed::prelude::*;

#[derive(Default, Debug)]
struct Pirrigator {
    weather: weather::Model,
    sensors: sensors::Model
}

#[derive(Clone)]
enum Message {
    Weather(weather::Message),
    Sensors(sensors::Message),
}

fn update(msg: Message, model: &mut Pirrigator) -> Update<Message> {
    match msg {
        Message::Weather(msg) => model.weather.update(msg).map(Message::Weather),
        Message::Sensors(msg) => model.sensors.update(msg).map(Message::Sensors)
    }
}

fn view(model: &Pirrigator) -> El<Message> {
    div![
        h1!["Pirrigator"],
        model.weather.render().map_message(Message::Weather),
        model.sensors.render().map_message(Message::Sensors)
    ]
}

#[wasm_bindgen]
pub fn render() {
    seed::App::build(Pirrigator::default(), update, view)
        .finish()
        .run();
}
