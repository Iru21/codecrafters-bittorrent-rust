mod parser;
mod torrent;
mod request;
mod connection;

use std::env;
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

        let response = TrackerRequest::new(&meta.info)
            .fetch_peers(&meta.announce);
        let peers = response.format_peers();
        let peer = peers[piece_index % peers.len()].clone();

        let mut connection = Connection::new(peer);
        connection.handshake(meta.info.hash().to_vec(), PEER_ID);

        // println!("* Handshake complete, waiting for bitfield, begining exchange");

        connection.wait(Connection::BITFIELD);

        connection.send_interested();
        connection.wait(Connection::UNCHOKE);

        // println!("* Unchoked, requesting piece {}", piece_index);

        connection.download_piece(meta, piece_index as u32, path);

    } else {
        println!("unknown command: {}", args[1])
    }
}