use std::{
    io::{BufRead, BufReader},
    path::PathBuf,
};

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
    pub records: Option<Vec<(String, Records)>>,
}
impl TableFile {
    pub fn load(&mut self) -> Result<()> {
        match self.path.extension().and_then(|ext| ext.to_str()) {
            Some("csv") => self.load_csv(),
            Some("csvs") => self.load_csvs(),
            _ => panic!("Unrecognized extension. Should be either `csv` or `csvs`."),
        }
    }

    fn load_csv(&mut self) -> Result<()> {
        let mut rdr = csv::Reader::from_path(&self.path)?;
        let headers = rdr.headers()?.clone();
        let rows: Vec<StringRecord> = rdr.into_records().filter_map(Result::ok).collect();
        let records = vec![(String::new(), Records { headers, rows })];
        self.records = Some(records);
        Ok(())
    }

    fn load_csvs(&mut self) -> Result<()> {
        let file = fs_err::File::open(&self.path)?;
        let reader = BufReader::new(file);

        let mut buffers: Vec<(String, Vec<String>)> = vec![];
        let mut buffer = vec![];
        let mut name = String::new();

        for line in reader.lines() {
            let line = line?;
            if line.starts_with("#>") {
                if !buffer.is_empty() {
                    buffers.push((name, buffer.drain(..).collect()));
                }
                name = line.chars().skip(2).take_while(|c| *c != ',').collect();
            } else {
                buffer.push(line);
            }
        }
        if !buffer.is_empty() {
            buffers.push((name, buffer.drain(..).collect()));
        }

        let records: Result<_> = buffers
            .into_iter()
            .map(|(name, buf)| {
                let buf = buf.join("\n").into_bytes();
                let mut rdr = csv::Reader::from_reader(buf.as_slice());
                let headers = rdr.headers()?.clone();
                let rows: Vec<StringRecord> = rdr.into_records().filter_map(Result::ok).collect();
                Ok((name, Records { headers, rows }))
            })
            .collect();
        self.records = Some(records?);
        Ok(())
    }

    pub fn n_sheets(&self) -> usize {
        self.records
            .as_ref()
            .map(|records| records.len())
            .unwrap_or_default()
    }
}
