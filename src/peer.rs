use std::fs::{self, File};
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::path::Path;
use serde::{Serialize, Deserialize};

const PEERS_FILE: &str = "data/peers.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Peer {
    pub address: String,
    pub port: u16,
}

impl Peer {
    pub fn socket_addr(&self) -> SocketAddr {
        format!("{}:{}", self.address, self.port)
            .parse()
            .expect("Invalid socket address")
    }
}

pub fn load_peers() -> Vec<Peer> {
    if Path::new(PEERS_FILE).exists() {
        let mut file = File::open(PEERS_FILE).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        serde_json::from_str(&contents).unwrap_or_default()
    } else {
        vec![]
    }
}

pub fn save_peers(peers: &[Peer]) {
    fs::create_dir_all("data").unwrap();
    let serialized = serde_json::to_string_pretty(peers).unwrap();
    let mut file = File::create(PEERS_FILE).unwrap();
    file.write_all(serialized.as_bytes()).unwrap();
}