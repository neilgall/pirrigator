# InfluxDB Client Library

[![version](https://img.shields.io/crates/v/influxc.svg?style=flat-square)](https://crates.io/crates/influxc)
[![docs](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/influxc)

## About this crate

### What this crate provides

- Support for InfluxDB 2.x.
- Backlog storage of Record's on failure to commit due to connectivity or configuration issues.
- Build-in compression of requests.

### What it does not provide

- Support for InfluxDB 1.x

### What is on the roadmap

- Support for async/await as a feature. [#3](https://github.com/voipir/rust-influxc/issues/3)
- Reduction of dependencies by switching the underlying reqwest library with hyper. [#4](https://github.com/voipir/rust-influxc/issues/4)
- Support for sending, processing responses to queries. [#5](https://github.com/voipir/rust-influxc/issues/5)
- Support for mapping native types to query response data like sqlx. [#6](https://github.com/voipir/rust-influxc/issues/6)

## Basic Usage

```rust
use influxc::Client;
use influxc::FileBacklog;

use influxc::Record;
use influxc::Precision;
use influxc::Credentials;
use influxc::InfluxError;

use std::time::Duration;
use std::thread::sleep;

fn main() -> Result<(), InfluxError>
{
    let creds   = Credentials::from_basic("testuser", "testpasswd");
    let backlog = FileBacklog::new("./ignore/backlog")?;

    let mut client = Client::build("http://127.0.0.1:8086".into(), creds)
        .backlog(backlog)
        .finish()?;

    let mut rec = Record::new("org", "bucket")
        .precision(Precision::Milliseconds);

    loop
    {
        rec.measurement("sensor1")
            .tag("floor", "second")
            .tag("exposure", "west")
            .field("temp", 123)
            .field("brightness", 500);

        rec.measurement("sensor2")
            .tag("floor", "second")
            .tag("exposure", "east")
            .field("temp", 321)
            .field("brightness", 999);

        if let Err(e) = client.write(&rec) {
            eprintln!("{}", e);
        }

        sleep(Duration::from_secs(1));
    }
}
```
