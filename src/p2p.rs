use std::{net::{TcpListener, TcpStream}, io::{Read, Write}, thread, sync::{Arc, Mutex}};
use crate::transaction::Transaction;

/// Simple message type: JSON of a Transaction or Block
#[derive(Debug, Clone)]
pub enum Message {
    Transaction(Transaction),
    Block(String), // TODO: full Block JSON
}

pub fn start_node(my_port: u16, peers: Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind(("0.0.0.0", my_port)).expect("bind failed");
    println!("P2P node listening on {}", my_port);

    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            let peers = peers.clone();
            thread::spawn(move || handle_connection(&mut stream, peers));
        }
    }
}

fn handle_connection(stream: &mut TcpStream, peers: Arc<Mutex<Vec<String>>>) {
    let mut buf = Vec::new();
    if stream.read_to_end(&mut buf).is_ok() {
        let msg = String::from_utf8_lossy(&buf);
        println!("[P2P] Received: {}", msg);
        // TODO: parse JSON, update mempool or chain
    }
}

/// Broadcast to all known peers
pub fn broadcast(peers: &Arc<Mutex<Vec<String>>>, msg: &str) {
    for peer in peers.lock().unwrap().iter() {
        if let Ok(mut s) = TcpStream::connect(peer) {
            let _ = s.write_all(msg.as_bytes());
        }
    }
}