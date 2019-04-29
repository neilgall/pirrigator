# Raspberry Pi Greenhouse Automation in Rust

This is my little project to automate watering my tomatoes while learning Rust. So far the electronics are built and I'm working on a modular code structure where each device runs in its own thread and communicates by message passing. I'm gathering all the sensor data in an sqlite database, from which I hope to draw graphs and serve over HTTP.

Right now it expects a couple of my other projects in the same parent directory:
* https://github.com/neilgall/rust-mcp3xxx
* https://github.com/neilgall/rust-bme280
