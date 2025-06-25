use abcy_data::schema::{parse_streams, ParsedStreams};
use serde_json::json;

#[test]
fn parse_time_stream() {
    let v = json!({"time": {"data": [1,2,3]}});
    let parsed = parse_streams(&v).unwrap();
    assert_eq!(parsed, ParsedStreams { time: vec![1,2,3] });
}
