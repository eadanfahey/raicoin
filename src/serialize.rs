use serde;
use serde_json;

pub fn serialize<T: serde::Serialize>(item: &T) -> String {
    serde_json::to_string(item).unwrap()
}

pub fn deserialize<'a, T: serde::Deserialize<'a>>(s: &'a str) -> T {
    serde_json::from_str(s).unwrap()
}
