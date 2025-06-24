use polars::prelude::*;
use std::fs::File;
use std::path::PathBuf;
use crate::fit_parser::DataPoint;

#[derive(Clone)]
pub struct Storage {
    pub data_dir: PathBuf,
}

impl Storage {
    pub fn new<P: Into<PathBuf>>(dir: P) -> Self {
        Self { data_dir: dir.into() }
    }

    pub fn has_activity(&self, activity_id: u64) -> bool {
        self.data_dir.join(format!("{}.parquet", activity_id)).exists()
    }

    pub fn save_activity(&self, activity_id: u64, points: &[DataPoint]) -> PolarsResult<()> {
        let timestamp: Vec<Option<i64>> = points.iter().map(|p| p.timestamp).collect();
        let power: Vec<Option<i64>> = points.iter().map(|p| p.power).collect();
        let hr: Vec<Option<i64>> = points.iter().map(|p| p.heart_rate).collect();
        let cadence: Vec<Option<i64>> = points.iter().map(|p| p.cadence).collect();
        let distance: Vec<Option<f64>> = points.iter().map(|p| p.distance).collect();

        let mut df = DataFrame::new(vec![
            Series::new("timestamp", timestamp),
            Series::new("power", power),
            Series::new("heart_rate", hr),
            Series::new("cadence", cadence),
            Series::new("distance", distance),
        ])?;
        let path = self.data_dir.join(format!("{}.parquet", activity_id));
        std::fs::create_dir_all(&self.data_dir)?;
        let file = File::create(path)?;
        ParquetWriter::new(file).finish(&mut df)?;
        Ok(())
    }
}
