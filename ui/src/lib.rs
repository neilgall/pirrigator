#[macro_use] extern crate serde_derive;
#[macro_use] extern crate seed;

mod chart;
mod zones;
mod weather;
mod utils;

use seed::prelude::*;

#[derive(Default, Debug)]
struct Pirrigator {
    weather: weather::Model,
    zones: zones::Model
}

#[derive(Clone)]
enum Message {
    Weather(weather::Message),
    Zones(zones::Message),
}

fn update(msg: Message, model: &mut Pirrigator) -> Update<Message> {
    match msg {
        Message::Weather(msg) => model.weather.update(msg).map(Message::Weather),
        Message::Zones(msg) => model.zones.update(msg).map(Message::Zones)
    }
}

fn view(model: &Pirrigator) -> El<Message> {
    div![
        h1!["Pirrigator"],
        model.weather.render().map_message(Message::Weather),
        model.zones.render().map_message(Message::Zones)
    ]
}

#[wasm_bindgen]
pub fn render() {
    seed::App::build(Pirrigator::default(), update, view)
        .finish()
        .run();
}
