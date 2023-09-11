mod parser;
mod torrent;
mod request;
mod connection;

use std::env;
use std::io::Write;
use torrent::{Torrent};
use sha1::{Sha1, Digest};
use crate::connection::Connection;
use crate::parser::{decode, ValueToString};
use crate::request::TrackerRequest;

const PEER_ID: &str = "00112233445566778899";

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

        let mut connection = Connection::new(peer);

        let res = connection.handshake(meta.info.hash().to_vec(), PEER_ID);

        let peer_id = res.peer_id.iter().map(|b| {
            format!("{:02x}", b)
        }).collect::<Vec<String>>().join("");

        println!("Peer ID: {}", peer_id);
    } else if command == "download_piece" {
        // I literally do not care enough to refactor this for clap, ughghgughhgh
        // syntax: command download_piece -o /path/ file.torrent index
        let meta = Torrent::from_file(&args[4]);
        let piece_index = args[5].parse::<usize>().unwrap();
        let path = args[3].clone();

        const CHUNK_SIZE: usize = 16 * 1024;

        let response = TrackerRequest::new(&meta.info)
            .fetch_peers(meta.announce);
        let peer = response.format_peers()[0].clone();

        let mut connection = Connection::new(peer);
        connection.handshake(meta.info.hash().to_vec(), PEER_ID);

        println!("Handshake complete, requesting piece {}", &piece_index);

        connection.send_interested();
        connection.wait(Connection::UNCHOKE);

        let block_count = meta.info.piece_length / CHUNK_SIZE;
        for i in 0..block_count {
            let length = if i == block_count - 1 {
                meta.info.piece_length - (i * CHUNK_SIZE)
            } else {
                CHUNK_SIZE
            };

            println!("Requesting block {} of length {}", i, length);
            connection.send_request(piece_index as u32, (i * CHUNK_SIZE) as u32, length as u32);
        }


        let mut piece_data = vec![0; meta.info.piece_length];
        for _ in 0..block_count {
            let resp = connection.wait(Connection::PIECE);
            println!("Received response of length {}", resp.len());
            let index = u32::from_be_bytes([resp[0], resp[1], resp[2], resp[3]]);
            if index != piece_index as u32 {
                println!("index mismatch, expected {}, got {}", &piece_index, index);
                continue;
            }

            let begin = u32::from_be_bytes([resp[4], resp[5], resp[6], resp[7]]) as usize;

            println!("Received block {} of length {}", begin / CHUNK_SIZE, resp.len() - 8);
            piece_data.splice(begin..begin + CHUNK_SIZE, resp[8..].iter().cloned());
        }

        println!("All pieces received, verifying hash");
        let mut hasher = Sha1::new();
        hasher.update(&piece_data.as_slice());
        let fetched_piece_hash = hasher.finalize().iter().map(|b| {
            format!("{:02x}", b)
        }).collect::<Vec<String>>().join("");

        let piece_hash = meta.info.pieces()[piece_index].clone();
        if fetched_piece_hash == piece_hash {
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .open(&path)
                .unwrap();

            file.write_all(&piece_data).unwrap();

            println!("Piece {} downloaded to {}", &piece_index, &path);
        } else {
            println!("piece hash mismatch, expected {}({}), got {}({})", piece_hash, piece_hash.len(), fetched_piece_hash, fetched_piece_hash.len());
        }

    } else {
        println!("unknown command: {}", args[1])
    }
}