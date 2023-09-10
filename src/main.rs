use std::{env, fs};
use serde::Deserialize;
use serde_bencode::{self, value::Value};

fn decode(encoded_value: &str) -> Value {
    return serde_bencode::from_str::<Value>(encoded_value).unwrap();
}

fn format(value: &Value) -> String {
    return match value {
        Value::Bytes(bytes) => format!("{:?}", std::str::from_utf8(bytes).unwrap()),
        Value::Int(i) => i.to_string(),
        Value::List(list) => format!("[{}]", list.iter().map(format).collect::<Vec<String>>().join(",")),
        Value::Dict(dict) => {
            let mut result = Vec::<String>::new();
            for (key, value) in dict {
                let key_str = String::from_utf8_lossy(key).to_string();

                result.push(format!("\"{}\":{}", key_str, format(value)));
            }
            result.sort();
            format!("{{{}}}", result.join(","))
        }
    }
}

#[derive(Debug, Deserialize)]
struct Info {
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    #[serde(rename = "piece length")]
    piece_length: usize,
    length: usize,
}

#[derive(Debug, Deserialize)]
struct MetaInfo {
    announce: String,
    info: Info
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];
        let decoded_value = decode(encoded_value);
        println!("{}", format(&decoded_value));
    } else if command == "info" {
        let data = fs::read(&args[2]).unwrap();
        let meta: MetaInfo = serde_bencode::from_bytes(&data).unwrap();
        println!("Tracker URL: {}", meta.announce);
        println!("Length: {}", meta.info.length);
    } else {
        println!("unknown command: {}", args[1])
    }
}
