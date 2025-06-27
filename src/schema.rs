use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityHeader {
    pub id: u64,
    pub name: String,
    pub start_date: String,
    pub distance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetail {
    pub meta: serde_json::Value,
    pub streams: ParsedStreams,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActivitySummary {
    pub id: u64,
    pub name: String,
    pub start_date: String,
    pub distance: f64,
    pub duration: i64,
    pub average_power: Option<f64>,
    /// Average speed in meters per second if available
    pub average_speed: Option<f64>,
    /// Number of personal records from segments if available
    pub pr_count: Option<i64>,
    /// Average heart rate in bpm if available
    pub average_heartrate: Option<f64>,
    /// Summary polyline of the activity map if available
    pub summary_polyline: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParsedStreams {
    pub time: Vec<i64>,
    /// Power data in watts if available
    pub power: Vec<i64>,
    /// Heart rate data in bpm if available
    pub heartrate: Vec<i64>,
}

pub fn parse_streams(v: &serde_json::Value) -> Option<ParsedStreams> {
    let time = v.get("time")?.get("data")?.as_array()?;
    let power = v
        .get("watts")
        .and_then(|p| p.get("data"))
        .and_then(|d| d.as_array())
        .map(|arr| arr.iter().map(|x| x.as_i64().unwrap_or(0)).collect())
        .unwrap_or_else(Vec::new);
    let heartrate = v
        .get("heartrate")
        .and_then(|p| p.get("data"))
        .and_then(|d| d.as_array())
        .map(|arr| arr.iter().map(|x| x.as_i64().unwrap_or(0)).collect())
        .unwrap_or_else(Vec::new);
    Some(ParsedStreams {
        time: time.iter().map(|x| x.as_i64().unwrap_or(0)).collect(),
        power,
        heartrate,
    })
}
