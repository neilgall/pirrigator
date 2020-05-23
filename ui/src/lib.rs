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

fn update(msg: Message, model: &mut Pirrigator, orders: &mut impl Orders<Message>) {
    match msg {
        Message::Weather(msg) => weather::update(msg, &mut model.weather, &mut orders.proxy(Message::Weather)),
        Message::Zones(msg) => zones::update(msg, &mut model.zones, &mut orders.proxy(Message::Zones))
    }
}

fn view(model: &Pirrigator) -> Node<Message> {
    div![
        h1!["Pirrigator"],
        weather::render(&model.weather).map_msg(Message::Weather),
        zones::render(&model.zones).map_msg(Message::Zones)
    ]
}

fn after_mount(_: Url, orders: &mut impl Orders<Message>) -> AfterMount<Pirrigator> {
    weather::after_mount(&mut orders.proxy(Message::Weather));
    zones::after_mount(&mut orders.proxy(Message::Zones));
    AfterMount::default()
}


#[wasm_bindgen]
pub fn render() {
    seed::App::builder(update, view)
        .after_mount(after_mount)
        .build_and_start();
}
