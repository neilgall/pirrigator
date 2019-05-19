# Raspberry Pi Greenhouse Automation in Rust

This is my little project to automate watering my tomatoes while learning Rust. So far we have
* gathering data via hardware sensors
* storing the data in an sqlite database
* serving the data via a web JSON API
* serving a WebAssembly UI to access this API and present the data
* a configuration of sensors/valves to "zones" and scheduled irrigation of zones

Still to do:
* guard irrigation using current moisture level in each zone
* UI for irrigation events
* push notifications

I recommend following my sequence of Pirrigator posts on [dev.to](https://dev.to/neilgall).