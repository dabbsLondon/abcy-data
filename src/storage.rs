use polars::prelude::*;
use crate::strava::ActivitySummary;
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

    pub fn fit_file_path(&self, activity_id: u64) -> PathBuf {
        self.data_dir.join("raw").join(format!("{}.fit", activity_id))
    }

    pub fn metadata_path(&self) -> PathBuf {
        self.data_dir.join("metadata.parquet")
    }

    pub fn has_fit_file(&self, activity_id: u64) -> bool {
        self.fit_file_path(activity_id).exists()
    }

    pub fn has_metadata(&self, activity_id: u64) -> bool {
        let path = self.metadata_path();
        if !path.exists() {
            return false;
        }
        if let Ok(df) = ParquetReader::new(File::open(path).unwrap()).finish() {
            if let Ok(series) = df.column("id") {
                if let Ok(ca) = series.u64() {
                    return ca.into_no_null_iter().any(|v| v == activity_id);
                }
            }
        }
        false
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

    pub fn save_metadata(&self, act: &ActivitySummary) -> PolarsResult<()> {
        let path = self.metadata_path();
        std::fs::create_dir_all(self.data_dir.join("raw"))?;
        let fit_file = self.fit_file_path(act.id).display().to_string();
        let mut new_df = df![
            "id" => [act.id],
            "name" => [act.name.as_str()],
            "start_date" => [act.start_date.as_str()],
            "distance" => [act.distance],
            "fit_file" => [fit_file.as_str()],
        ]?;
        if path.exists() {
            let mut df = ParquetReader::new(File::open(&path)?).finish()?;
            if let Ok(series) = df.column("id") {
                if let Ok(ca) = series.u64() {
                    if ca.into_no_null_iter().any(|v| v == act.id) {
                        return Ok(());
                    }
                }
            }
            df.vstack_mut(&new_df)?;
            let file = File::create(&path)?;
            ParquetWriter::new(file).finish(&mut df)?;
        } else {
            let file = File::create(&path)?;
            ParquetWriter::new(file).finish(&mut new_df)?;
        }
        Ok(())
    }
}
