use abcy_data::schema::{parse_streams, ParsedStreams};
use serde_json::json;

#[test]
fn parse_time_stream() {
    let v = json!({"time": {"data": [1,2,3]}});
    let parsed = parse_streams(&v).unwrap();
    assert_eq!(
        parsed,
        ParsedStreams {
            time: vec![1, 2, 3],
            power: vec![],
        }
    );
}

#[test]
fn parse_power_stream() {
    let v = json!({"time": {"data": [0]}, "watts": {"data": [100, 200]}});
    let parsed = parse_streams(&v).unwrap();
    assert_eq!(parsed.power, vec![100, 200]);
}
