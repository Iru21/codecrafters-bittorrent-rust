use std::fs;
use serde::{Deserialize, Serialize};
use sha1::digest::Output;
use sha1::{Sha1, Digest};

#[derive(Debug, Deserialize, Serialize)]
pub struct TorrentInfo {
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    #[serde(rename = "piece length")]
    pub piece_length: usize,
    pub length: usize,
    #[serde(with = "serde_bytes")]
    pub pieces: Vec<u8>,
}

impl TorrentInfo {
    pub fn bytes(&self) -> Vec<u8> {
        return serde_bencode::to_bytes(&self).unwrap();
    }

    pub fn hash(&self) -> Output<Sha1> {
        let mut hasher = Sha1::new();
        hasher.update(self.bytes());
        return hasher.finalize();
    }

    pub fn hex_hash(&self) -> String {
        return format!("{:x}", self.hash());
    }

    pub fn url_encoded_hash(&self) -> String {
        self.hash().iter().map(|b| {
            format!("%{:02x}", b)
        }).collect::<Vec<String>>().join("")
    }

    pub fn pieces(&self) -> Vec<String> {
        return self.pieces.chunks(20).map(|chunk| {
            format!("{}", chunk.iter().map(|b| {
                format!("{:02x}", b)
            }).collect::<Vec<String>>().join("")
            )
        }).collect();
    }
}

#[derive(Debug, Deserialize)]
pub struct Torrent {
    pub announce: String,
    pub info: TorrentInfo
}

impl Torrent {
    pub fn from_file(path: &str) -> Torrent {
        let data = fs::read(path).unwrap();
        return serde_bencode::from_bytes(&data).unwrap();
    }
}