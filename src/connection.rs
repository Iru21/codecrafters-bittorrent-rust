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
}