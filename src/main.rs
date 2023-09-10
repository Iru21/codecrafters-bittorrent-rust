use std::{env, fs};
use serde::{Deserialize, Serialize};
use serde_bencode::{self, value::Value};
use sha1::{Digest, Sha1};

fn decode(encoded_value: &str) -> Value {
    return serde_bencode::from_str::<Value>(encoded_value).unwrap();
}

trait ValueToString {
    fn to_string(&self) -> String;
}

impl ValueToString for Value {
    fn to_string(&self) -> String {
        return match self {
            Value::Bytes(bytes) => format!("{:?}", std::str::from_utf8(bytes).unwrap()),
            Value::Int(i) => i.to_string(),
            Value::List(list) => format!("[{}]", list.iter().map(|v| { v.to_string() }).collect::<Vec<String>>().join(",")),
            Value::Dict(dict) => {
                let mut result = Vec::<String>::new();
                for (key, value) in dict {
                    let key_str = String::from_utf8_lossy(key).to_string();

                    result.push(format!("\"{}\":{}", key_str, value.to_string()));
                }
                result.sort();
                format!("{{{}}}", result.join(","))
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct TorrentInfo {
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    #[serde(rename = "piece length")]
    piece_length: usize,
    length: usize,
    #[serde(with = "serde_bytes")]
    pieces: Vec<u8>,
}

impl TorrentInfo {
    fn hash(&self) -> String {
        let bencoded_info = serde_bencode::to_bytes(self).unwrap();
        let mut hasher = Sha1::new();
        hasher.update(bencoded_info);
        return format!("{:x}", hasher.finalize());
    }
}

#[derive(Debug, Deserialize)]
struct Torrent {
    announce: String,
    info: TorrentInfo
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];
        let decoded_value = decode(encoded_value);
        println!("{}", decoded_value.to_string());
    } else if command == "info" {
        let data = fs::read(&args[2]).unwrap();
        let meta: Torrent = serde_bencode::from_bytes(&data).unwrap();

        println!("Tracker URL: {}", meta.announce);
        println!("Length: {}", meta.info.length);
        println!("Info Hash: {}", meta.info.hash());
    } else {
        println!("unknown command: {}", args[1])
    }
}
