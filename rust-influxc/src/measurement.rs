//!
//! Measurement to be Stored
//!
use crate::Value;
use crate::Precision;

use crate::Utc;
use crate::DateTime;

use std::collections::BTreeMap;


/// The smallest unit of recording. Multiple of these Measurements are fit in a [Record](struct.Record.html), which in
/// turn is submitted to InfluxDB.
#[derive(Debug, Deserialize, Serialize)]
pub struct Measurement
{
    pub(crate) name: String,

    pub(crate) tags:   BTreeMap<String, String>,
    pub(crate) fields: BTreeMap<String, Value>,

    pub(crate) timestamp: DateTime,
}


impl Measurement
{
    pub(crate) fn new(name: &str) -> Self
    {
        Self {
            name: name.to_owned(),

            tags:   BTreeMap::new(),
            fields: BTreeMap::new(),

            // TODO stamp in Drop of a MeasurementBuilder taking reference to collection in Record
            timestamp: Utc::now(),
        }
    }

    /// Set datetime of this Measurement
    pub fn timestamp(&mut self, timestamp: DateTime) -> &mut Self
    {
        self.timestamp = timestamp; self
    }

    /// Add a tag to this Measurement
    pub fn tag(&mut self, key: &str, value: &str) -> &mut Self
    {
        self.tags.insert(key.to_owned(), value.to_owned());
        self
    }

    /// Add a value field to this Measurement
    pub fn field<V: Into<Value>>(&mut self, key: &str, value: V) -> &mut Self
    {
        self.fields.insert(key.to_owned(), value.into());
        self
    }

    pub(crate) fn to_line(&self, precision: &Precision) -> String
    {
        let mut line = self.name.to_owned();

        if ! self.tags.is_empty()
        {
            let tagline = self.tags.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<String>>()
                .join(",");

            line += ",";
            line += &tagline;
        }

        if ! self.fields.is_empty()
        {
            let fieldline = self.fields.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<String>>()
                .join(",");

            line += " ";
            line += &fieldline;
        }

        line += " ";

        match precision
        {
            Precision::Nanoseconds  => { line += &self.timestamp.timestamp_nanos().to_string();  }
            Precision::Microseconds => { line += &(self.timestamp.timestamp_nanos() * 1000).to_string(); }
            Precision::Milliseconds => { line += &self.timestamp.timestamp_millis().to_string(); }
            Precision::Seconds      => { line += &(self.timestamp.timestamp() ).to_string(); }
        }

        line
    }
}
