#[macro_use] extern crate serde_derive;
#[macro_use] extern crate seed;
extern crate chrono;

mod weather;
mod utils;

use seed::prelude::*;
use weather::Weather;

#[derive(Default, Debug)]
struct Pirrigator {
    weather: Weather,
}

#[derive(Clone)]
enum Message {
    Weather(weather::Message)
}

fn update(msg: Message, model: &mut Pirrigator) -> Update<Message> {
    match msg {
        Message::Weather(msg) => {
            model.weather.update(msg).map(Message::Weather);
            Render.into()
        }
    }
}

fn view(model: &Pirrigator) -> El<Message> {
    div![
        h1!["Pirrigator"],
        model.weather.render().map_message(Message::Weather)
    ]
}

#[wasm_bindgen]
pub fn render() {
    seed::App::build(Pirrigator::default(), update, view)
        .finish()
        .run();
}
