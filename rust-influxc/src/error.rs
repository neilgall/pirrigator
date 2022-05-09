//!
//! Error Handling
//!
use crate::JsonError;

use crate::ReqwError;

use crate::Deserialize;


pub(crate) type InfluxResult<T> = Result<T, InfluxError>;


pub(crate) trait InfluxErrorAnnotate<T>
{
    fn annotate<M: ToString>(self, msg: M) -> InfluxResult<T>;
}


/// ## Chaining Support
///
/// Project wide enumeration of errors that this library emits. If you have your own error type, you might want to chain
/// this error into it like so:
///
/// ```rust
/// use influxc::InfluxError;
///
/// #[derive(Debug)]
/// enum MyError
/// {
///     Influx(InfluxError)
/// }
///
/// impl From<InfluxError> for MyError {
///     fn from(other: InfluxError) -> Self {
///         Self::Influx(other)
///     }
/// }
///
/// impl std::fmt::Display for MyError {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///         match self {
///             Self::Influx(ref other) => write!(f, "{}", other)
///         }
///     }
/// }
///
/// impl std::error::Error for MyError {
///     fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
///         match self {
///             Self::Influx(ref other) => Some(other)
///         }
///     }
/// }
/// ```
#[derive(Debug)]
pub enum InfluxError
{
    /// Internal error message
    Error(String),

    /// Annotated error message. Allows for context providing in case of an error.
    Annotated(String, Box<InfluxError>),

    /// Chaining of [IoError](std::io::Error)
    Io(std::io::Error),

    /// Chaining of [ParseBoolError](std::str::ParseBoolError)
    ParseBool(std::str::ParseBoolError),

    /// Chaining of [JsonError](serde_json::error::Error)
    Json(JsonError),

    /// Chaining of [ReqwestError](reqwest::error::Error)
    Reqwest(ReqwError),

    /// Authentication API: Not authorized (log-in missing) error.
    AuthUnauthorized(ApiGenericError),

    /// Authentication API: Account is currently disabled. Check
    AuthAccountDisabled(ApiGenericError),

    /// Authentication API: Unknown credentials. Invalid username/passwd in basic?. Invalid token?
    AuthUnknown(ApiGenericError),

    /// Write API: Malformed write request. Something is not properly formatted for InfluxDB. Please report bug.
    WriteMalformed(ApiMalformationError),

    /// Write API: Not authorized to write to that bucket. Check permissions in InfluxDB GUI.
    WriteUnauthorized(ApiGenericError),

    /// Write API: Not yet authenticated for write. Authenticate first.
    WriteUnauthenticated(ApiGenericError),

    /// Write API: Request is to big in size. Reduce the amount of measurements in submitted record.
    WriteOversized(ApiOversizeError),

    /// Write API: Request limit reached. Try again later.
    WriteOverquota(ApiDelayError),

    /// Write API: InfluxDB currently not ready. Try again later.
    WriteUnready(ApiDelayError),

    /// Write API: InfluxDB server side error. Investigate.
    WriteUnknown(ApiGenericError),
}


#[derive(Debug, Deserialize)]
pub struct ApiGenericError
{
    code:    String,
    message: String,
}


#[derive(Debug, Deserialize)]
pub struct ApiDelayError
{
    delay: i64,
}


#[derive(Debug, Deserialize)]
pub struct ApiMalformationError
{
    code:    String,
    err:     Option<String>,
    line:    Option<i32>,
    message: String,
    op:      Option<String>,
}


#[derive(Debug, Deserialize)]
pub struct ApiOversizeError
{
    code: String,

    #[serde(rename="maxLength")]
    maxlen:  i32,

    message: String,
}


impl<T, E> InfluxErrorAnnotate<T> for Result<T, E>
    where E: Into<InfluxError> + std::error::Error
{
    fn annotate<M: ToString>(self, msg: M) -> InfluxResult<T>
    {
        self.map_err(|e| {
            InfluxError::Annotated(msg.to_string(), Box::new(e.into()))
        })
    }
}


impl From<&str>   for InfluxError { fn from(err: &str)   -> InfluxError { InfluxError::Error(err.to_owned()) }}
impl From<String> for InfluxError { fn from(err: String) -> InfluxError { InfluxError::Error(err) }}

impl From<std::io::Error>           for InfluxError { fn from(err: std::io::Error)           -> InfluxError { InfluxError::Io(err) }}
impl From<std::str::ParseBoolError> for InfluxError { fn from(err: std::str::ParseBoolError) -> InfluxError { InfluxError::ParseBool(err) }}

impl From<JsonError> for InfluxError { fn from(err: JsonError) -> InfluxError { InfluxError::Json(err) }}
impl From<ReqwError> for InfluxError { fn from(err: ReqwError) -> InfluxError { InfluxError::Reqwest(err) }}


impl std::fmt::Display for InfluxError
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        match *self
        {
            Self::Error(ref err)        => { write!(f, "{}", err) }
            Self::Annotated(ref msg, _) => { write!(f, "{}", msg) },

            Self::Io(ref err)        => { write!(f, "Io Error: {}",      err) }
            Self::ParseBool(ref err) => { write!(f, "Parse Bool Error: {}",      err) }

            Self::Json(ref err)       => { write!(f, "Json Error: {}",    err) }
            Self::Reqwest(ref err)    => { write!(f, "Reqwest Error: {}", err) }

            Self::AuthUnauthorized(ref inner)     => { write!(f, "AuthUnauthorized({})",     inner) }
            Self::AuthAccountDisabled(ref inner)  => { write!(f, "AuthAccountDisabled({})",  inner) }
            Self::AuthUnknown(ref inner)          => { write!(f, "AuthUnknown({})",          inner) }
            Self::WriteMalformed(ref inner)       => { write!(f, "WriteMalformed({})",       inner) }
            Self::WriteUnauthorized(ref inner)    => { write!(f, "WriteUnauthorized({})",    inner) }
            Self::WriteUnauthenticated(ref inner) => { write!(f, "WriteUnauthenticated({})", inner) }
            Self::WriteOversized(ref inner)       => { write!(f, "WriteOversized({})",       inner) }
            Self::WriteOverquota(ref inner)       => { write!(f, "WriteOverquota({})",       inner) }
            Self::WriteUnready(ref inner)         => { write!(f, "WriteUnready({})",         inner) }
            Self::WriteUnknown(ref inner)         => { write!(f, "WriteUnknown({})",         inner) }
        }
    }
}


impl std::fmt::Display for ApiGenericError
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        write!(f, "code={}, message={}", self.code, self.message)
    }
}


impl std::fmt::Display for ApiDelayError
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        write!(f, "delay={}s", self.delay)
    }
}


impl std::fmt::Display for ApiMalformationError
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        write!(f, "code={}, err={}, line={}, message={}, op={}",
            self.code,
            self.err.as_ref().map(|v| v.to_string()).unwrap_or_else(|| "n/a".to_owned()),
            self.line.map(|v| v.to_string()).unwrap_or_else(|| "n/a".to_owned()),
            self.message,
            self.op.as_ref().map(|v| v.to_string()).unwrap_or_else(|| "n/a".to_owned())
        )
    }
}


impl std::fmt::Display for ApiOversizeError
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        write!(f, "code={}, maxlen={}, message={}", self.code, self.maxlen, self.message)
    }
}


impl std::error::Error for InfluxError
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)>
    {
        match *self
        {
            InfluxError::Error(_)              => { None }
            InfluxError::Annotated(_, ref err) => { Some(err) }

            InfluxError::Io(ref err)        => { Some(err) }
            InfluxError::ParseBool(ref err) => { Some(err) }

            InfluxError::Json(ref err)    => { Some(err) }
            InfluxError::Reqwest(ref err) => { Some(err) }

            InfluxError::AuthUnauthorized(_)     => { None }
            InfluxError::AuthAccountDisabled(_)  => { None }
            InfluxError::AuthUnknown(_)          => { None }
            InfluxError::WriteMalformed(_)       => { None }
            InfluxError::WriteUnauthorized(_)    => { None }
            InfluxError::WriteUnauthenticated(_) => { None }
            InfluxError::WriteOversized(_)       => { None }
            InfluxError::WriteOverquota(_)       => { None }
            InfluxError::WriteUnready(_)         => { None }
            InfluxError::WriteUnknown(_)         => { None }
        }
    }
}
