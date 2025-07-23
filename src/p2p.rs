use crate::blockchain::Blockchain;
use std::{
    io::Read,
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
};

/// Start a simple P2P listener (stub for real sync)
pub fn start_node(port: u16, blockchain: Arc<Mutex<Blockchain>>, peers: Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind(("0.0.0.0", port)).expect("P2P bind failed");
    println!("🌐 P2P listening on port {}", port);
    for stream in listener.incoming() {
        if let Ok(mut s) = stream {
            handle_conn(&mut s);
        }
    }
}

fn handle_conn(stream: &mut TcpStream) {
    let mut buf = [0; 1024];
    if let Ok(n) = stream.read(&mut buf) {
        println!("P2P recv: {}", String::from_utf8_lossy(&buf[..n]));
    }
}