use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::io::{Read, Write};
use crate::blockchain::Blockchain;

pub fn start_node(port: u16, blockchain: Arc<Mutex<Blockchain>>, peers: Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind(("0.0.0.0", port)).expect("P2P bind failed");
    println!("🌐 P2P node listening on port {}", port);

    for stream in listener.incoming() {
        match stream {
            Ok(mut s) => {
                let chain = blockchain.clone();
                let peer_list = peers.clone();
                std::thread::spawn(move || handle_connection(&mut s, chain, peer_list));
            }
            Err(e) => {
                println!("❌ Connection error: {}", e);
            }
        }
    }
}

fn handle_connection(stream: &mut TcpStream, blockchain: Arc<Mutex<Blockchain>>, _peers: Arc<Mutex<Vec<String>>>) {
    let mut buffer = [0u8; 512];
    if let Ok(size) = stream.read(&mut buffer) {
        let received = String::from_utf8_lossy(&buffer[..size]);
        println!("🌐 P2P message received: {}", received);

        if received.trim() == "GET_BLOCKCHAIN" {
            let chain = blockchain.lock().unwrap();
            let json = serde_json::to_string(&*chain).unwrap_or_else(|_| "[]".to_string());
            let _ = stream.write_all(json.as_bytes());
        }
    }
}