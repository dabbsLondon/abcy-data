use fitparser::{self, Value, profile::MesgNum};
use serde::Serialize;
use std::fs::File;
use std::path::Path;
use std::convert::TryInto;

#[derive(Debug, Serialize)]
pub struct DataPoint {
    pub timestamp: Option<i64>,
    pub power: Option<i64>,
    pub heart_rate: Option<i64>,
    pub cadence: Option<i64>,
    pub distance: Option<f64>,
}

pub fn parse_fit_file(path: &Path) -> anyhow::Result<Vec<DataPoint>> {
    let mut file = File::open(path)?;
    let records = fitparser::from_reader(&mut file)?;
    let mut points = Vec::new();
    for record in records {
        if record.kind() == MesgNum::Record {
            let mut point = DataPoint { timestamp: None, power: None, heart_rate: None, cadence: None, distance: None };
            for field in record.into_vec() {
                match field.name() {
                    "timestamp" => if let Ok(v) = <Value as TryInto<i64>>::try_into(field.into_value()) { point.timestamp = Some(v); },
                    "power" => if let Ok(v) = <Value as TryInto<i64>>::try_into(field.into_value()) { point.power = Some(v); },
                    "heart_rate" => if let Ok(v) = <Value as TryInto<i64>>::try_into(field.into_value()) { point.heart_rate = Some(v); },
                    "cadence" => if let Ok(v) = <Value as TryInto<i64>>::try_into(field.into_value()) { point.cadence = Some(v); },
                    "distance" => if let Ok(v) = <Value as TryInto<f64>>::try_into(field.into_value()) { point.distance = Some(v); },
                    _ => {}
                }
            }
            points.push(point);
        }
    }
    Ok(points)
}
