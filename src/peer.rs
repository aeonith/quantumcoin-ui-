use crate::blockchain::{Block, Blockchain};
use crate::transaction::Transaction;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;
use std::fs;

pub fn broadcast_transaction(tx: &Transaction) {
    if let Ok(peer_list) = fs::read_to_string("peers.json") {
        if let Ok(peers) = serde_json::from_str::<Vec<String>>(&peer_list) {
            for peer in peers {
                if let Ok(mut stream) = TcpStream::connect(&peer) {
                    let message = format!("TRANSACTION:{}", serde_json::to_string(tx).unwrap());
                    let _ = stream.write_all(message.as_bytes());
                }
            }
        }
    }
}

pub fn broadcast_block(block: &Block) {
    if let Ok(peer_list) = fs::read_to_string("peers.json") {
        if let Ok(peers) = serde_json::from_str::<Vec<String>>(&peer_list) {
            for peer in peers {
                if let Ok(mut stream) = TcpStream::connect(&peer) {
                    let message = format!("BLOCK:{}", serde_json::to_string(block).unwrap());
                    let _ = stream.write_all(message.as_bytes());
                }
            }
        }
    }
}

pub fn start_peer_server(blockchain: &mut Blockchain) {
    let listener = TcpListener::bind("0.0.0.0:6001").expect("Peer server failed to bind");
    println!("[P2P] Peer node listening on port 6001...");

    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            let mut buffer = [0; 2048];
            if let Ok(size) = stream.read(&mut buffer) {
                let msg = String::from_utf8_lossy(&buffer[..size]);
                if msg.starts_with("TRANSACTION:") {
                    if let Ok(tx) = serde_json::from_str::<Transaction>(&msg["TRANSACTION:".len()..]) {
                        println!("[P2P] Received TX from peer");
                        blockchain.add_transaction(tx);
                    }
                } else if msg.starts_with("BLOCK:") {
                    if let Ok(block) = serde_json::from_str::<Block>(&msg["BLOCK:".len()..]) {
                        println!("[P2P] Received BLOCK from peer");
                        blockchain.chain.push(block);
                    }
                }
            }
        }
    }
}