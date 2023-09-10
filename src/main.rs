use std::env;
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
            let mut result = String::from("{");
            for (key, value) in dict {
                result.push_str(&format!("\"{}\": {}", std::str::from_utf8(key).unwrap(), format(value)));

                if key != dict.keys().last().unwrap() {
                    result.push_str(", ");
                }
            }
            result.push_str("}");
            result
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];
        let decoded_value = decode(encoded_value);
        println!("{}", format(&decoded_value));
    } else {
        println!("unknown command: {}", args[1])
    }
}
