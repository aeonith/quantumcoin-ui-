use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::blockchain::Blockchain;
use crate::transaction::Transaction;

pub fn start_peer_server(blockchain: Arc<Mutex<Blockchain>>, port: u16) {
    let listener = TcpListener::bind(("0.0.0.0", port)).expect("Failed to bind peer port");

    println!("🛰️ Listening for peers on port {}", port);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let chain_clone = Arc::clone(&blockchain);
                thread::spawn(move || {
                    handle_peer_connection(stream, chain_clone);
                });
            }
            Err(e) => eprintln!("Peer connection error: {:?}", e),
        }
    }
}

fn handle_peer_connection(mut stream: TcpStream, blockchain: Arc<Mutex<Blockchain>>) {
    let mut buffer = Vec::new();
    if stream.read_to_end(&mut buffer).is_ok() {
        if let Ok(text) = String::from_utf8(buffer.clone()) {
            if text.starts_with("BLOCK") {
                let block_data = text.replacen("BLOCK:", "", 1);
                if let Ok(new_block) = serde_json::from_str(&block_data) {
                    let mut chain = blockchain.lock().unwrap();
                    chain.try_add_block(new_block);
                }
            } else if text.starts_with("TX") {
                let tx_data = text.replacen("TX:", "", 1);
                if let Ok(tx) = serde_json::from_str::<Transaction>(&tx_data) {
                    let mut chain = blockchain.lock().unwrap();
                    chain.add_transaction(tx);
                }
            }
        }
    }
}

pub fn broadcast_transaction(tx: &Transaction, peers: &[String]) {
    let serialized = format!("TX:{}", serde_json::to_string(tx).unwrap());
    for peer in peers {
        if let Ok(mut stream) = TcpStream::connect(peer) {
            let _ = stream.write_all(serialized.as_bytes());
        }
    }
}

pub fn broadcast_block<B: serde::Serialize>(block: &B, peers: &[String]) {
    let serialized = format!("BLOCK:{}", serde_json::to_string(block).unwrap());
    for peer in peers {
        if let Ok(mut stream) = TcpStream::connect(peer) {
            let _ = stream.write_all(serialized.as_bytes());
        }
    }
}