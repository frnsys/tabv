use std::path::PathBuf;

use color_eyre::Result;
use csv::StringRecord;

#[derive(Debug)]
pub struct Records {
    pub headers: StringRecord,
    pub rows: Vec<StringRecord>,
}

#[derive(Debug)]
pub struct TableFile {
    pub name: String,
    pub path: PathBuf,
    pub records: Option<Records>,
}
impl TableFile {
    pub fn load(&mut self) -> Result<()> {
        let mut rdr = csv::Reader::from_path(&self.path)?;
        let headers = rdr.headers()?.clone();
        let rows: Vec<StringRecord> = rdr.into_records().filter_map(Result::ok).collect();
        self.records = Some(Records { headers, rows });
        Ok(())
    }
}
