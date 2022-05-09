//!
//! Persistant side storage for failed inserts "after the fact" as important distinction to a "write ahead log"
//! thus aiming to be more flash friendly on embedded devices where you want to keep writes to a minimum. This
//! will cause data loss if after failed insert also the writing to flash fails!. For that prefer a WAL approach.
//!
//! WAL approach: TODO
//!
use super::Backlog;

use crate::Record;
use crate::Precision;

use crate::InfluxError;
use crate::InfluxErrorAnnotate;
use crate::InfluxResult;

use crate::b32;
use crate::json;

use std::fs::File;
use std::fs::OpenOptions;

use std::io::Seek;
use std::io::SeekFrom;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::io::BufWriter;

use std::path::Path;
use std::path::PathBuf;

use std::collections::HashMap;


/// Backlog for the [Client](struct.Client.html) to persist [Record](struct.Record.html)`s that could not be submitted
/// to the InfluxDB due to conectivity or configuration errors.
#[derive(Debug)]
pub struct FileBacklog
{
    dir: PathBuf,

    archives: HashMap<PathBuf, Archive>,
}


impl FileBacklog
{
    /// Construct file based backlog by providing a directory to write [Record](struct.Record.html) files to.
    pub fn new<P: AsRef<Path>>(dir: P) -> InfluxResult<Self>
    {
        let dir = PathBuf::from(dir.as_ref());

        if ! dir.is_dir() {
            return Err(format!("Backlog dir is not a directory: {:#?}", dir).into())
        }

        let mut archives = HashMap::new();

        let listing = std::fs::read_dir(&dir)
            .annotate(format!("While opening backlog directory: {:#?}", dir))?;

        for entry in listing
        {
            let entry = entry?;
            let path  = entry.path();
            let file  = Archive::open(&path)?;

            archives.insert(path, file);
        }

        Ok(Self {dir, archives})
    }

    fn archive(&mut self, record: &Record) -> InfluxResult<&mut Archive>
    {
        let meta = ArchiveMeta::from_record(record);
        let path = meta.to_path();

        if ! self.archives.contains_key(&path) {
            self.archives.insert(path.clone(), Archive::open(&path)?);
        }

        Ok(self.archives.get_mut(&path).unwrap())
    }
}


impl Backlog for FileBacklog
{
    #[inline]
    fn read_pending(&mut self) -> InfluxResult<Vec<Record>>
    {
        let mut records = Vec::new();

        for archive in self.archives.values_mut()
        {
            if let Some(record) = archive.record()? {
                records.push(record);
            }
        }

        Ok(records)
    }

    #[inline]
    fn write_pending(&mut self, record: &Record) -> InfluxResult<()>
    {
        self.archive(record)?
            .append(record)?;

        Ok(())
    }

    #[inline]
    fn truncate_pending(&mut self, record: &Record) -> InfluxResult<()>
    {
        self.archive(record)?
            .truncate()?;

        Ok(())
    }
}


#[derive(Debug)]
struct Archive
{
    meta:   ArchiveMeta,
    handle: Option<File>,
    count:  usize,
}


impl Archive
{
    pub fn open(path: &Path) -> InfluxResult<Self>
    {
        let meta   = ArchiveMeta::from_path(&path)?;
        let handle = open(&path, false)?;
        let bfrd   = BufReader::new(&handle);
        let count  = bfrd.lines().count();

        Ok(Self {meta, handle: Some(handle), count})
    }

    pub fn record(&mut self) -> InfluxResult<Option<Record>>
    {
        if self.count == 0 {
            Ok(None)
        }
        else
        {
            self.prepare_handle(Some(SeekFrom::Start(0)))?;

            if let Some(handle) = &self.handle
            {
                let reader = BufReader::new(handle);

                let mut msrmts = Vec::new();

                for (num, line) in reader.lines().enumerate()
                {
                    let ln = line?;

                    match json::from_str(&ln)
                    {
                        Ok(msrmt) => {
                            msrmts.push(msrmt)
                        }

                        Err(e) => {
                            error!("Failed to read line {}", num);
                            return Err(e.into());
                        }
                    }
                }

                let mut record = Record::new(&self.meta.org, &self.meta.bucket)
                    .precision(self.meta.precision.clone());

                record.measurements = msrmts;

                Ok(Some(record))
            }
            else {
                panic!("handle preparation should have prevented this case");
            }
        }

    }

    pub fn append(&mut self, record: &Record) -> InfluxResult<()>
    {
        self.prepare_handle(Some(SeekFrom::End(0)))?;

        if let Some(handle) = &self.handle
        {
            let mut writer = BufWriter::new(handle);

            for msrmt in record.measurements.iter()
            {
                let line = json::to_string(msrmt)?;

                writer.write_all(line.as_bytes())?;
                writer.write_all(b"\n")?;

                self.count += 1;
            }

            writer.flush()?;
        }
        else {
            panic!("handle preparation should have prevented this case");
        }

        Ok(())
    }

    pub fn truncate(&mut self) -> InfluxResult<()>
    {
        std::fs::remove_file(&self.meta.to_path())?;  // to keep dir as clean as possible from empty backlogs

        self.handle = None;
        self.count  = 0;

        Ok(())
    }

    fn prepare_handle(&mut self, seek: Option<SeekFrom>) -> InfluxResult<()>
    {
        if self.handle.is_none() {
            self.handle = Some(open(&self.meta.to_path(), false)?);
        }

        let handle = self.handle.as_mut().unwrap();

        if let Some(pos) = seek {
            handle.seek(pos)?;
        }

        Ok(())
    }
}


fn open(path: &Path, truncate: bool) -> InfluxResult<File>
{
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(truncate)
        .open(path)
        .annotate(format!("While opening file: {:#?}", path))
}


#[derive(Debug)]
struct ArchiveMeta
{
    org:       String,  // TODO Cow<'s, &'s str>
    bucket:    String,  // TODO Cow<'s, &'s str>
    precision: Precision,
}


impl ArchiveMeta
{
    fn from_record(record: &Record) -> Self
    {
        Self {
            org:       record.org.clone(),
            bucket:    record.bucket.clone(),
            precision: record.precision.clone(),
        }
    }

    fn from_path(path: &Path) -> InfluxResult<Self>
    {
        let stem = path.file_stem()
            .ok_or_else::<InfluxError, _>(|| format!("Could not extract file stem from: {:#?}", path).into())?;

        let dec32 = b32::decode(b32::Alphabet::RFC4648 {padding: false}, stem.to_str().unwrap())
            .ok_or_else::<InfluxError, _>(|| format!("Could not base32 decode file name for its parts: {:#?}", path).into())?;

        let name = String::from_utf8(dec32)
            .map_err(|e| InfluxError::Error(format!("Invalid UTF8 while decoding archive path '{:#?}': {}", path, e)))?;

        let parts = name.split('_')
            .collect::<Vec<&str>>();

        if parts.len() < 3 {
            Err(format!("Could not determine archive name from path: {:#?}", path).into())
        } else {
            let org       = parts[0].to_owned();
            let bucket    = parts[1].to_owned();
            let precision = parts[2].parse()?;

            Ok(ArchiveMeta {org, bucket, precision})
        }
    }

    fn to_path(&self) -> PathBuf
    {
        let name  = format!("{}_{}_{}", self.org, self.bucket, self.precision.to_string());
        let enc32 = b32::encode(b32::Alphabet::RFC4648 {padding: false}, name.as_bytes());

        PathBuf::from(format!("{}.log", enc32))
    }
}
