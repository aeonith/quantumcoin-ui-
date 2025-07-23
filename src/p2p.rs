use crate::blockchain::Blockchain;
use std::{
    io::Read,
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
};

pub fn start_node(port: u16, blockchain: Arc<Mutex<Blockchain>>, peers: Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind(("0.0.0.0", port)).expect("P2P bind failed");
    println!("🌐 P2P listening on port {}", port);

    for stream in listener.incoming() {
        if let Ok(mut s) = stream {
            handle_connection(&mut s, &blockchain, &peers);
        }
    }
}

fn handle_connection(stream: &mut TcpStream, _chain: &Arc<Mutex<Blockchain>>, _peers: &Arc<Mutex<Vec<String>>>) {
    let mut buf = Vec::new();
    if stream.read_to_end(&mut buf).is_ok() {
        println!("🌐 P2P recv: {}", String::from_utf8_lossy(&buf));
    }
}