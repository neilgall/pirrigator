//!
//! Client Builder
//!
use crate::Client;
use crate::Credentials;

use crate::Backlog;
use crate::NoopBacklog;

use crate::InfluxResult;


/// Builder to piece by piece assemble a [Client](struct.Client.html) instance
pub struct ClientBuilder
{
    url:   String,
    creds: Credentials,

    backlog: Option<Box<dyn Backlog>>
}


impl ClientBuilder
{
    /// Create builder with the most basic information necessary
    pub fn new(url: String, creds: Credentials) -> Self
    {
        Self {
            url, creds,

            backlog: None,
        }
    }

    /// Add backlog to client, so records and measurements get stored as log as
    /// they fail to be committed. Either due to conectivity or misconfiguration.
    pub fn backlog<B: Backlog + 'static>(mut self, backlog: B) -> Self
    {
        self.backlog = Some(Box::new(backlog)); self
    }

    /// Consume this builder to assemble and return the final Client instance
    /// for usage.
    pub fn finish(self) -> InfluxResult<Client>
    {
        let backlog = match self.backlog
        {
            Some(b) => { b }
            None    => { Box::new(NoopBacklog::new()) }
        };

        Client::new(self.url, self.creds, backlog)
    }
}
