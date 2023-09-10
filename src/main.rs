use std::env;
use serde_bencode::{self, value::Value};


fn format(value: &Value) -> String {
    return match value {
        Value::Bytes(bytes) => format!("{:?}", std::str::from_utf8(bytes).unwrap()),
        Value::Int(i) => i.to_string(),
        Value::List(list) => list.iter().map(format).collect::<Vec<String>>().join(","),
        _ => panic!("invalid type"),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];
        let decoded_value = serde_bencode::from_str::<Value>(encoded_value).unwrap();
        println!("{}", format(&decoded_value));
    } else {
        println!("unknown command: {}", args[1])
    }
}
