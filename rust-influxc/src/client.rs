//!
//! Client Connection and Interface to Database
//!
use crate::Record;
use crate::Credentials;
use crate::ClientBuilder;

use crate::Backlog;

use crate::InfluxError;
use crate::InfluxResult;

use crate::ApiDelayError;
use crate::ApiGenericError;
use crate::ApiOversizeError;
use crate::ApiMalformationError;

use crate::b64;

use crate::ReqwUrl;
use crate::ReqwClient;
use crate::ReqwMethod;
use crate::ReqwRequestBuilder;


/// The basic unit of interactino with the InfluxDB API.
#[derive(Debug)]
pub struct Client
{
    url:    ReqwUrl,
    creds:  Credentials,
    client: ReqwClient,

    backlog: Box<dyn Backlog>,
}


impl Client
{
    /// Create a builder to parametrize and construct this [Client](struct.Client.html).
    pub fn build(url: String, creds: Credentials) -> ClientBuilder
    {
        ClientBuilder::new(url, creds)
    }

    /// Directly construct this [Client](struct.Client.html).
    pub fn new(url: String, creds: Credentials, backlog: Box<dyn Backlog>) -> InfluxResult<Self>
    {
        // let ignore_cert = std::env::var("INFLUX_UNSAFE_TLS").ok()
        //     .unwrap_or_else(|| "false".to_owned())
        //     .parse()?;

        let client = ReqwClient::builder()
//            .danger_accept_invalid_certs(ignore_cert)
            .build()?;

        let url = match ReqwUrl::parse(&url)
        {
            Ok(url) => { url }
            Err(e)  => { return Err(format!("Failed to parse URL: {} due to {}", url, e).into()) }
        };

        let mut this = Self {client, url, creds, backlog};

        this.authenticate()?;

        Ok(this)
    }

    /// Submit a [Record](struct.Record.html) to be written to InfluxDB. Or backlogged if you set a backlogger.
    pub fn write(&mut self, record: &Record) -> InfluxResult<()>
    {
        if let Err(e) = self.write_backlog() {
            self.backlog.write_pending(&record)?; Err(e)
        }
        else
        {
            let result = self.write_record(&record);

            if result.is_err() {
                self.backlog.write_pending(&record)?;
            }

            result
        }
    }

    /// Submit pending/backlogged [Records](struct.Record.html) to writing. It will attempt to flush them to database.
    pub fn flush(&mut self) -> InfluxResult<()>
    {
        self.write_backlog()
    }
}


/// Private interface
impl Client
{
    fn write_backlog(&mut self) -> InfluxResult<()>
    {
        let records = self.backlog.read_pending()?;

        for record in records.iter()
        {
            info!("Found {} backlogged entries, attempting to commit", records.len());

            if let Err(e) = self.write_record(&record) {
                return Err(InfluxError::Error(format!("Unable to commit backlogged record: {}", e)));
            }
            else
            {
                let result = self.backlog.truncate_pending(&record);

                if let Err(e) = result
                {
                    let msg = format!("Failed to eliminate/truncate record from backlog: {}", e);
                    error!("{}", msg);
                    panic!("{}", msg);
                }
                else {
                    return Ok(());
                }
            }
        }

        Ok(())
    }

    fn write_record(&self, record: &Record) -> InfluxResult<()>
    {
        let mut url = self.url.clone();

        url.set_path("/api/v2/write");

        let mut builder = self.client.request(ReqwMethod::POST, url);

        builder = record.to_write_request(builder)?;
        builder = self.inject_credentials(builder)?;

        debug!("Request: {:#?}", builder);

        let reply = builder.send()?;

        match reply.status().as_u16()
        {
            204 => { info!("Written: {}", record); Ok(()) }

            400 => { Err(InfluxError::WriteMalformed(reply.json::<ApiMalformationError>()?)) }
            401 => { Err(InfluxError::WriteUnauthorized(reply.json::<ApiGenericError>()?)) }
            403 => { Err(InfluxError::WriteUnauthenticated(reply.json::<ApiGenericError>()?)) }
            413 => { Err(InfluxError::WriteOversized(reply.json::<ApiOversizeError>()?)) }
            429 => { Err(InfluxError::WriteOverquota(reply.json::<ApiDelayError>()?)) }
            503 => { Err(InfluxError::WriteUnready(reply.json::<ApiDelayError>()?)) }

            _   => { Err(InfluxError::WriteUnknown(reply.json::<ApiGenericError>()?)) }
        }
    }

    fn authenticate(&mut self) -> InfluxResult<()>
    {
        if let Credentials::Basic{ref user, ref passwd, cookie: None} = self.creds
        {
            let mut url = self.url.clone();

            url.set_path("/api/v2/signin");

            let b64creds = b64::encode(format!("{}:{}", user, passwd));

            let req = self.client.request(ReqwMethod::POST, url)
                .header("Authorization", format!("Basic {}", b64creds));

            debug!("Request: {:#?}", req);

            let rep = req.send()?;

            match rep.status().as_u16()
            {
                204 => {
                    if let Some(cookie) = rep.headers().get("Set-Cookie")
                    {
                        let session = {
                            if let Ok(s) = cookie.to_str() {
                                s.to_owned()
                            } else {
                                return Err(format!("Failed to extract session cookie string: {:#?}", cookie).into());
                            }
                        };

                        self.creds = Credentials::Basic {user: user.clone(), passwd: passwd.clone(), cookie: Some(session)};
                    }
                    else {
                        return Err("Missing session cookie after successfull basic auth".into());
                    }
                }

                401 => { return Err(InfluxError::AuthUnauthorized(rep.json::<ApiGenericError>()?)); }
                403 => { return Err(InfluxError::AuthAccountDisabled(rep.json::<ApiGenericError>()?)); }
                _   => { return Err(InfluxError::AuthUnknown(rep.json::<ApiGenericError>()?)); }
            }
        }

        Ok(())
    }

    fn inject_credentials(&self, builder: ReqwRequestBuilder) -> InfluxResult<ReqwRequestBuilder>
    {
        match &self.creds
        {
            Credentials::Basic{user: _, passwd: _, cookie: None} => {
                Err("Missing session cookie from basic auth. This should not have happened!".into())
            }

            Credentials::Basic{user: _, passwd: _, cookie: Some(session)} => {
                Ok(builder.header("Cookie", session))
            }

            Credentials::Token{token} => {
                Ok(builder.header("Authorization", format!("Token {}", token)))
            }
        }
    }
}
