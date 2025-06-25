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
    pub streams: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParsedStreams {
    pub time: Vec<i64>,
}

pub fn parse_streams(v: &serde_json::Value) -> Option<ParsedStreams> {
    let time = v.get("time")?.get("data")?.as_array()?;
    Some(ParsedStreams { time: time.iter().map(|x| x.as_i64().unwrap_or(0)).collect() })
}
