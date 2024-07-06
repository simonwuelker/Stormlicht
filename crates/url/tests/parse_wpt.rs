use std::fs;

use serialize::Deserialize;
use serialize_json::{JsonDeserializer, Value};

pub const TESTCASES: &str = concat!(env!("TEST_DIR"), "/wpt/url/resources/urltestdata.json");

#[test]
fn wpt_url_parse_tests() {
    // Load the WPT test cases
    let file_data =
        fs::read(TESTCASES).expect("Testcases could not be read, did you download the submodules?");
    let json_blob = String::from_utf8(file_data).expect("urltestdata.json contains non-utf8 data");
    let parsed_json = Value::deserialize(&mut JsonDeserializer::new(&json_blob))
        .expect("urltestdata.json contains invalid json");

    let test_cases = parsed_json
        .as_list()
        .expect("expected a list of testcases")
        .map(|case| case.as_map().expect("test case should be a map"));

    // Actually run the tests
    for test_case in test_cases {
        panic!("{:?}", test_case);
    }
}
