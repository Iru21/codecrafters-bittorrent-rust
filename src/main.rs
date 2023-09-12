mod parser;
mod torrent;
mod request;
mod connection;

use std::env;
use std::io::Write;
use torrent::{Torrent};
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
            .fetch_peers(&meta.announce);

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
                .fetch_peers(&meta.announce);

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

        let peers = TrackerRequest::new(&meta.info)
            .fetch_peers(&meta.announce).format_peers();
        let peer = peers[0].clone();

        let mut connection = Connection::new(peer);
        connection.prepare(meta.info.hash().to_vec(), PEER_ID);
        let data = connection.download_piece(meta, piece_index as u32);
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&path)
            .unwrap();

        file.write_all(&data).unwrap();

        println!("Piece {} downloaded to {}.", &piece_index, &path);
    } else if command == "download" {
        // syntax: command download -o /path/ file.torrent
        let meta = Torrent::from_file(&args[4]);
        let path = args[3].clone();

        let peers = TrackerRequest::new(&meta.info)
            .fetch_peers(&meta.announce).format_peers();

        let mut torrent = vec![0; meta.info.length];
        for (i, _) in meta.info.pieces().iter().enumerate() {
            let peer = peers[0].clone();

            let mut connection = Connection::new(peer);
            connection.prepare(meta.info.hash().to_vec(), PEER_ID);
            let piece_data = connection.download_piece(meta.clone(), i as u32);
            torrent.splice(i * meta.info.piece_length..i * meta.info.piece_length + piece_data.len(), piece_data.iter().cloned());
        }
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&path)
            .unwrap();

        file.write_all(&torrent).unwrap();

        println!("Downloaded {} to {}.", &args[4], &path);
    } else {
        println!("unknown command: {}", args[1])
    }
}