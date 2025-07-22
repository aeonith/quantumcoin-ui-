use std::net::{TcpListener, TcpStream};
use std::io::{BufRead, BufReader, Write};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::blockchain::Blockchain;
use crate::wallet::Wallet;
use crate::peer::{Peer, load_peers, save_peers};
use crate::transaction::Transaction;
use crate::block::Block;

pub fn start_networking(wallet: Arc<Wallet>, blockchain: Arc<Mutex<Blockchain>>) {
    let listener = TcpListener::bind("0.0.0.0:7878").expect("Could not bind to port 7878");
    println!("🌐 Listening on port 7878 for peers...");

    // Accept incoming connections
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let chain = Arc::clone(&blockchain);
                thread::spawn(move || {
                    handle_connection(stream, chain);
                });
            }
            Err(e) => {
                eprintln!("❌ Connection failed: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream, blockchain: Arc<Mutex<Blockchain>>) {
    let peer_addr = stream.peer_addr().unwrap();
    println!("🤝 Connected: {}", peer_addr);

    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut line = String::new();

    while let Ok(bytes) = reader.read_line(&mut line) {
        if bytes == 0 {
            break;
        }

        if line.starts_with("BLOCK:") {
            let json = line.trim_start_matches("BLOCK:");
            if let Ok(block) = serde_json::from_str::<Block>(json) {
                let mut chain = blockchain.lock().unwrap();
                if block.previous_hash == chain.latest_block().hash {
                    println!("📦 Received valid block from peer.");
                    chain.chain.push(block);
                    chain.save_to_disk();
                }
            }
        } else if line.starts_with("TX:") {
            let json = line.trim_start_matches("TX:");
            if let Ok(tx) = serde_json::from_str::<Transaction>(json) {
                println!("💸 Received transaction from peer.");
                blockchain.lock().unwrap().add_transaction(tx);
            }
        }

        line.clear();
    }
}

pub fn broadcast_transaction(tx: &Transaction) {
    let peers = load_peers();
    let serialized = serde_json::to_string(tx).unwrap();
    let msg = format!("TX:{}\n", serialized);

    for peer in peers {
        if let Ok(mut stream) = TcpStream::connect(peer.socket_addr()) {
            let _ = stream.write_all(msg.as_bytes());
        }
    }
}

pub fn broadcast_block(block: &Block) {
    let peers = load_peers();
    let serialized = serde_json::to_string(block).unwrap();
    let msg = format!("BLOCK:{}\n", serialized);

    for peer in peers {
        if let Ok(mut stream) = TcpStream::connect(peer.socket_addr()) {
            let _ = stream.write_all(msg.as_bytes());
        }
    }
}

pub fn add_peer(ip: &str, port: u16) {
    let mut peers = load_peers();
    if !peers.iter().any(|p| p.address == ip && p.port == port) {
        peers.push(Peer {
            address: ip.to_string(),
            port,
        });
        save_peers(&peers);
        println!("🌍 Peer added: {}:{}", ip, port);
    }
}