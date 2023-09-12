use std::io::{Read, Write};
use std::net::TcpStream;
use sha1::{Sha1, Digest};
use crate::torrent::Torrent;

pub struct HandshakeResponse {
    pub info_hash: Vec<u8>,
    pub peer_id: Vec<u8>,
}

pub struct Connection {
    stream: TcpStream,
}

impl Connection {

    pub const UNCHOKE: u8 = 1;
    pub const INTERESTED: u8 = 2;
    pub const BITFIELD: u8 = 5;
    pub const REQUEST: u8 = 6;
    pub const PIECE: u8 = 7;

    pub fn new(peer: String) -> Self {
        let stream = TcpStream::connect(peer).unwrap();

        Connection {
            stream
        }
    }

    pub fn handshake(&mut self, info_hash: Vec<u8>, peer_id: &str) -> HandshakeResponse {
        let mut handshake = Vec::<u8>::new();

        handshake.push(19);
        handshake.extend(b"BitTorrent protocol");
        handshake.extend(vec![0; 8]);
        handshake.extend(info_hash);
        handshake.extend(peer_id.as_bytes().to_vec());

        self.stream.write_all(&handshake).unwrap();

        let mut handshake_response = [0; 68];
        self.stream.read_exact(&mut handshake_response).unwrap();

        HandshakeResponse {
            info_hash: handshake_response[28..48].to_vec(),
            peer_id: handshake_response[48..68].to_vec(),
        }
    }

    pub fn send_request(&mut self, index: u32, begin: u32, length: u32) {
        let mut payload = vec![0; 12];
        payload[0..4].copy_from_slice(&index.to_be_bytes());
        payload[4..8].copy_from_slice(&begin.to_be_bytes());
        payload[8..12].copy_from_slice(&length.to_be_bytes());

        self.send_message(Connection::REQUEST, payload);
    }

    pub fn send_interested(&mut self) {
        self.send_message(Connection::INTERESTED, vec![]);
    }

    pub fn send_message(&mut self, id: u8, payload: Vec<u8>) {
        let mut message = vec![0; 5 + payload.len()];
        let mut length = payload.len() as u32;
        if length == 0 {
            length = 1;
        }
        message[0..4].copy_from_slice(&(length).to_be_bytes());
        message[4] = id;
        message[5..].copy_from_slice(&payload);

        self.stream.write_all(&message).unwrap();
    }

    pub fn download_piece(&mut self, meta: Torrent, piece_index: u32, path: String) {
        let is_last_piece = piece_index as usize == meta.info.pieces().len() - 1;
        let piece_length = if is_last_piece {
            meta.info.length - (piece_index as usize * meta.info.piece_length)
        } else {
            meta.info.piece_length
        };
        println!("* Piece length: {}", piece_length);

        const CHUNK_SIZE: usize = 16 * 1024;
        let block_count = piece_length / CHUNK_SIZE + (piece_length % CHUNK_SIZE != 0) as usize;
        for i in 0..block_count {
            println!("++ Requesting block {}", i);
            let length = if i == block_count - 1 {
                piece_length - (i * CHUNK_SIZE)
            } else {
                CHUNK_SIZE
            };
            self.send_request(piece_index, (i * CHUNK_SIZE) as u32, length as u32);
        }


        let mut piece_data = vec![0; piece_length];
        for i in 0..block_count {
            let resp = self.wait(Connection::PIECE);
            println!("* Received response of length {} for block {}", resp.len(), i);
            let index = u32::from_be_bytes([resp[0], resp[1], resp[2], resp[3]]);
            if index != piece_index {
                println!("index mismatch, expected {}, got {}", &piece_index, index);
                continue;
            }

            let begin = u32::from_be_bytes([resp[4], resp[5], resp[6], resp[7]]) as usize;
            piece_data.splice(begin..begin + resp[8..].len(), resp[8..].iter().cloned());
        }

        println!("% All pieces received, verifying hash");
        let mut hasher = Sha1::new();
        hasher.update(&piece_data.as_slice());
        let fetched_piece_hash = hasher.finalize().iter().map(|b| {
            format!("{:02x}", b)
        }).collect::<Vec<String>>().join("");

        let piece_hash = meta.info.pieces()[piece_index as usize].clone();
        if fetched_piece_hash == piece_hash {
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .open(&path)
                .unwrap();

            file.write_all(&piece_data).unwrap();

            println!("Piece {} downloaded to {}.", &piece_index, &path);
        } else {
            println!("% piece hash mismatch, expected {}({}), got {}({})", piece_hash, piece_hash.len(), fetched_piece_hash, fetched_piece_hash.len());
        }
    }

    pub fn wait(&mut self, id: u8) -> Vec<u8> {
        let mut length_prefix = [0; 4];
        self.stream.read_exact(&mut length_prefix).expect("Failed to read length prefix");

        let mut message_id = [0; 1];
        self.stream.read_exact(&mut message_id).expect("Failed to read message id");

        if message_id[0] != id {
            panic!("* Expected message id {}, got {}", id, message_id[0]);
        }

        let resp_size = u32::from_be_bytes(length_prefix) - 1;
        let mut payload = vec![0; resp_size as usize];
        self.stream.read_exact(&mut payload).expect("Failed to read payload");

        payload
    }
}