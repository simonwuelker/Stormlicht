use std::fs;

use serialize::deserialization::Deserializer;
use serialize_json::{JsonDeserializer, Value};
use url::URL;

pub const WPT_TESTCASES: &str = concat!(env!("TEST_DIR"), "/wpt/url/resources/urltestdata.json");

#[derive(Debug)]
struct Error;

fn main() -> Result<(), Error> {
    let json_data = fs::read_to_string(WPT_TESTCASES).expect("wpt testcases not found");
    let wpt_data: Vec<Value> = JsonDeserializer::new(&json_data)
        .deserialize()
        .expect("wpt testcases contain invalid json");

    let test_cases = wpt_data.iter().flat_map(|value| value.as_map());

    let mut test_cases_succeeded = 0;
    let mut test_cases_run = 0;
    for test_case in test_cases {
        // Refer to https://github.com/web-platform-tests/wpt/tree/master/url for a description of the format
        let input = test_case.get("input").unwrap().as_str().unwrap();
        let base_str = match test_case.get("base").unwrap() {
            Value::Null => None,
            Value::String(s) => Some(s.as_str()),
            other => panic!("invalid base: {other:?}"),
        };
        let base = base_str.map(|s| s.parse().unwrap());
        let url = URL::parse_with_base(input, base.as_ref(), None);
        let mut succeeded = true;

        // Start test output
        print!("{:?}", input.escape_debug().collect::<String>());
        if let Some(base_str) = base_str {
            print!(
                " with base {:?}",
                base_str.escape_debug().collect::<String>()
            );
        }
        if test_case.contains_key("failure") {
            print!(" (should fail)");
        }
        print!(": ");

        if test_case.contains_key("failure") {
            succeeded = url.is_err();
        } else {
            let href = test_case.get("href").unwrap().as_str().unwrap();
            let origin = test_case.get("origin").map(|v| v.as_str().unwrap());
            let protocol = test_case.get("protocol").unwrap().as_str().unwrap();
            let username = test_case.get("username").unwrap().as_str().unwrap();
            let password = test_case.get("password").unwrap().as_str().unwrap();
            let host = test_case.get("host").unwrap().as_str().unwrap();
            let hostname = test_case.get("hostname").unwrap().as_str().unwrap();
            let port = test_case.get("port").unwrap().as_str().unwrap();
            let pathname = test_case.get("pathname").unwrap().as_str().unwrap();

            match url {
                Ok(url) => {
                    // FIXME compare all the values here
                    succeeded &= url.scheme() == &protocol[..protocol.len() - 1];
                    succeeded &= url.username() == username;
                    succeeded &= url.password() == password;
                    succeeded &= url.port().map(|p| p.to_string()).unwrap_or_default() == port;

                    let _ = href;
                    let _ = origin;
                    let _ = host;
                    let _ = hostname;
                    let _ = pathname;
                },
                Err(_) => {
                    succeeded = false;
                },
            }
        }

        if succeeded {
            println!("✅");
            test_cases_succeeded += 1;
        } else {
            println!("❌");
        }

        test_cases_run += 1;
    }

    println!("{test_cases_succeeded} out of {test_cases_run} test cases succeeded");

    if test_cases_run != test_cases_succeeded {
        Err(Error)
    } else {
        Ok(())
    }
}
