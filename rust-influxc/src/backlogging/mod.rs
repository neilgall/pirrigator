//!
//! Data Constistency Guarantee Methods.
//!
//! These are designed to provide the same interface than the client, wrapping around it and providing the same
//! interface `ClientTrait`.
//!
mod file;

use std::fmt::Debug;

pub use file::FileBacklog;

use crate::Record;
use crate::InfluxResult;


/// API definition that any backlog service needs to abide by so the [Client](struct.Client.html) can use it.
pub trait Backlog: Debug + Send + Sync
{
    /// Return any pending records that sits in backlog and requires to be commited.
    fn read_pending(&mut self) -> InfluxResult<Vec<Record>>;

    /// Write records that could not be commited, so they get written into backlog for future processing.
    fn write_pending(&mut self, record: &Record) -> InfluxResult<()>;

    /// Empty backlog from pending records. This gets called once all pending records have been
    /// successfully commited.
    fn truncate_pending(&mut self, record: &Record) -> InfluxResult<()>;
}


/// Backlog that does nothing
#[derive(Debug)]
pub struct NoopBacklog;


impl NoopBacklog
{
    /// Construct this dummy backlog that does nothing
    pub fn new() -> Self
    {
        Self {}
    }
}


impl Backlog for NoopBacklog
{
    fn read_pending(&mut self) -> InfluxResult<Vec<Record>> {
        Ok(Vec::new())
    }

    fn write_pending(&mut self, _: &Record) -> InfluxResult<()> {
        Ok(())
    }

    fn truncate_pending(&mut self, _: &Record) -> InfluxResult<()> {
        Ok(())
    }
}
