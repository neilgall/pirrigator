//!
//! InfluxDB Value Variants
//!
use std::fmt;


/// Type primitives as supported by InfluxDB and their conversions from/to Rust primitives
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value
{
    /// Self explanatory integer type
    #[serde(rename="i64")] Integer(i64),

    /// Self explanatory float type
    #[serde(rename="f64")] Float(f64),

    /// Self explanatory string type
    #[serde(rename="str")] String(String),

    /// Self explanatory boolean type
    #[serde(rename="bool")] Boolean(bool),

}


impl fmt::Display for Value
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        match self
        {
            Value::Integer(v) => v.fmt(f),
            Value::Float(v)   => v.fmt(f),
            Value::String(v)  => v.fmt(f),
            Value::Boolean(v) => v.fmt(f),
        }
    }
}


impl From<i64>    for Value { fn from(other: i64)    -> Self { Value::Integer(other) }}
impl From<f64>    for Value { fn from(other: f64)    -> Self { Value::Float(other) }}
impl From<String> for Value { fn from(other: String) -> Self { Value::String(other) }}
impl From<bool>   for Value { fn from(other: bool)   -> Self { Value::Boolean(other) }}
