use std::io::{Read, Write};
use std::net::TcpStream;

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

        // self.wait(Connection::BITFIELD);

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

    pub fn wait(&mut self, id: u8) -> Vec<u8> {
        let mut length_prefix = [0; 4];
        self.stream.read_exact(&mut length_prefix).expect("Failed to read length prefix");

        let mut message_id = [0; 1];
        self.stream.read_exact(&mut message_id).expect("Failed to read message id");

        if message_id[0] != id {
            panic!("Expected message id {}, got {}", id, message_id[0]);
        }

        let resp_size = u32::from_be_bytes(length_prefix) - 1;
        println!("Expecting {} bytes", resp_size);
        if resp_size == 0 {
            return vec![];
        }
        let mut payload = vec![0; resp_size as usize];
        self.stream.read_exact(&mut payload).expect("Failed to read payload");

        payload
    }
}