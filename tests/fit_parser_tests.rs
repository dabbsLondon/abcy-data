use abcy_data::fit_parser::parse_fit_file;
use std::path::Path;

#[test]
fn parse_nonexistent_file_returns_err() {
    let path = Path::new("not_real.fit");
    let res = parse_fit_file(path);
    assert!(res.is_err());
}
