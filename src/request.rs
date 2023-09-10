use serde::{Deserialize, Serialize};
use crate::PEER_ID;
use crate::torrent::TorrentInfo;

#[derive(Debug, Serialize, Deserialize)]
pub struct TrackerRequest {
    pub info_hash: String,
    pub peer_id: String,
    pub port: u16,
    pub uploaded: u64,
    pub downloaded: u64,
    pub left: u64,
    pub compact: u8,
}

impl TrackerRequest {
    pub fn new(meta: &TorrentInfo) -> Self {
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

    pub fn fetch_peers(&self, tracker_url: String) -> TrackerResponse {
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
pub struct TrackerResponse {
    pub interval: u64,
    #[serde(with = "serde_bytes")]
    pub peers: Vec<u8>,
}

impl TrackerResponse {
    pub fn format_peers(&self) -> Vec<String> {
        self.peers.chunks(6).map(|chunk| {
            let ip = chunk[0..4].iter().map(|b| {
                format!("{}", b)
            }).collect::<Vec<String>>().join(".");

            let port = u16::from_be_bytes([chunk[4], chunk[5]]);

            format!("{}:{}", ip, port)
        }).collect()
    }
}