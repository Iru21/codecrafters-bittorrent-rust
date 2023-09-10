use std::{env, fs};
use std::io::{Read, Write};
use std::net::TcpStream;
use serde::{Deserialize, Serialize};
use serde_bencode::{self, value::Value};
use sha1::{Digest, Sha1};
use sha1::digest::Output;

const PEER_ID: &str = "00112233445566778899";

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
    fn bytes(&self) -> Vec<u8> {
        return serde_bencode::to_bytes(&self).unwrap();
    }

    fn hash(&self) -> Output<Sha1> {
        let mut hasher = Sha1::new();
        hasher.update(self.bytes());
        return hasher.finalize();
    }

    fn hex_hash(&self) -> String {
        return format!("{:x}", self.hash());
    }

    fn url_encoded_hash(&self) -> String {
        self.hash().iter().map(|b| {
            format!("%{:02x}", b)
        }).collect::<Vec<String>>().join("")
    }

    fn pieces(&self) -> Vec<String> {
        return self.pieces.chunks(20).map(|chunk| {
            format!("{}", chunk.iter().map(|b| {
                    format!("{:02x}", b)
                }).collect::<Vec<String>>().join("")
            )
        }).collect();
    }
}

#[derive(Debug, Deserialize)]
struct Torrent {
    announce: String,
    info: TorrentInfo
}

impl Torrent {
    fn from_file(path: &str) -> Torrent {
        let data = fs::read(path).unwrap();
        return serde_bencode::from_bytes(&data).unwrap();
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct TrackerRequest {
    info_hash: String,
    peer_id: String,
    port: u16,
    uploaded: u64,
    downloaded: u64,
    left: u64,
    compact: u8,
}

impl TrackerRequest {
    fn new(meta: &TorrentInfo) -> Self {
        TrackerRequest {
            info_hash : meta.url_encoded_hash(),
            peer_id: PEER_ID.to_string(),
            port: 6881,
            uploaded: 0,
            downloaded: 0,
            left: meta.length as u64,
            compact: 1,
        }
    }

    fn fetch_peers(&self, tracker_url: String) -> TrackerResponse {
        let client = reqwest::blocking::Client::new();

        let url = format!("{}?info_hash={}", tracker_url, &self.info_hash);

        let req = client.get(&url)
            .query(&[("peer_id", self.peer_id.clone())])
            .query(&[("port", self.port.to_string())])
            .query(&[("uploaded", self.uploaded.to_string())])
            .query(&[("downloaded", self.downloaded.to_string())])
            .query(&[("left", self.left.to_string())])
            .query(&[("compact", self.compact.to_string())])
            .build().unwrap();

        let res = client.execute(req).unwrap();

        return serde_bencode::from_bytes(res.bytes().unwrap().as_ref()).unwrap()
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct TrackerResponse {
    interval: u64,
    #[serde(with = "serde_bytes")]
    peers: Vec<u8>,
}

impl TrackerResponse {
    fn format_peers(&self) -> Vec<String> {
        self.peers.chunks(6).map(|chunk| {
            let ip = chunk[0..4].iter().map(|b| {
                format!("{}", b)
            }).collect::<Vec<String>>().join(".");

            let port = u16::from_be_bytes([chunk[4], chunk[5]]);

            format!("{}:{}", ip, port)
        }).collect()
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];
        let decoded_value = decode(encoded_value);
        println!("{}", decoded_value.to_string());
    } else if command == "info" {
        let meta = Torrent::from_file(&args[2]);

        println!("Tracker URL: {}", meta.announce);
        println!("Length: {}", meta.info.length);
        println!("Info Hash: {}", meta.info.hex_hash());
        println!("Piece Length: {}", meta.info.piece_length);
        println!("Pieces:");

        for piece in meta.info.pieces() {
            println!("{}", piece);
        }
    } else if command == "peers" {
        let meta = Torrent::from_file(&args[2]);

        let response = TrackerRequest::new(&meta.info)
            .fetch_peers(meta.announce);

        println!("Peers:");

        for peer in response.format_peers() {
            println!("{}", peer);
        }
    } else if command == "handshake" {
        let meta = Torrent::from_file(&args[2]);

        let peer = if args.len() > 3 {
            args[3].clone()
        } else {
            let response = TrackerRequest::new(&meta.info)
                .fetch_peers(meta.announce);

            response.format_peers()[0].clone()
        };

        let mut stream = TcpStream::connect(peer).unwrap();

        let mut handshake = Vec::<u8>::new();
        handshake.push(19);
        handshake.append(&mut b"BitTorrent protocol".to_vec());
        handshake.append(&mut vec![0; 8]);
        handshake.append(&mut meta.info.hash().to_vec());
        handshake.append(&mut PEER_ID.as_bytes().to_vec());

        stream.write_all(handshake.as_slice()).unwrap();

        let mut handshake_response = [0; 68];
        stream.read_exact(&mut handshake_response).unwrap();

        // convert bytes to hex string
        let peer_id = handshake_response[48..68].iter().map(|b| {
            format!("{:02x}", b)
        }).collect::<Vec<String>>().join("");

        println!("Peer ID: {}", peer_id);

    } else {
        println!("unknown command: {}", args[1])
    }
}