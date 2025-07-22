use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    pub index: u64,
    pub data: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Blockchain {
    pub chain: Vec<Block>,
}

impl Blockchain {
    pub fn new() -> Self {
        let genesis_block = Block {
            index: 0,
            data: String::from("Genesis Block"),
        };
        Blockchain {
            chain: vec![genesis_block],
        }
    }

    // This must exist and be `pub`:
    pub fn load_from_file() -> Self {
        let path = "blockchain.json";
        if Path::new(path).exists() {
            let data = fs::read_to_string(path).expect("Failed to read blockchain file");
            serde_json::from_str(&data).expect("Failed to parse blockchain file")
        } else {
            let blockchain = Blockchain::new();
            let data = serde_json::to_string_pretty(&blockchain).expect("Failed to serialize blockchain");
            fs::write(path, data).expect("Failed to write blockchain file");
            blockchain
        }
    }
}