//!
//! Precision of the Measurement being Stored/Loaded
//!
use crate::InfluxError;

use std::fmt;


/// The time resolution the bucket is to keep its measurements
#[derive(Debug, Clone)]
pub enum Precision
{
    /// Self explanatory nanoseconds
    Nanoseconds,

    /// Self explanatory microseconds
    Microseconds,

    /// Self explanatory milliseconds
    Milliseconds,

    /// Self explanatory seconds
    Seconds,
}


impl std::str::FromStr for Precision
{
    type Err = InfluxError;

    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        let p = match s
        {
            "ns" => Precision::Nanoseconds,
             "u" => Precision::Microseconds,
            "ms" => Precision::Milliseconds,
             "s" => Precision::Seconds,

            _ => { return Err(format!("Invalid precision: {}", s).into()) }
        };

        Ok(p)
    }
}


impl fmt::Display for Precision
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        match self
        {
            Precision::Nanoseconds  => "ns".fmt(f),
            Precision::Microseconds =>  "u".fmt(f),
            Precision::Milliseconds => "ms".fmt(f),
            Precision::Seconds      =>  "s".fmt(f),
        }
    }
}


impl Default for Precision
{
    fn default() -> Self
    {
        Self::Nanoseconds
    }
}
