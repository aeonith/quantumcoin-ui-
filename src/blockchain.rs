use crate::wallet::Wallet;
use crate::revstop::is_revstop_active;
use crate::transaction::{Transaction, Block};
use std::fs::File;
use std::io::{Read, Write};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
}

impl Blockchain {
    pub fn new(genesis_recipient: &str) -> Self {
        let genesis = Block::genesis(genesis_recipient);
        Blockchain { blocks: vec![genesis] }
    }

    pub fn load_from_file() -> Option<Self> {
        let mut file = File::open("blockchain.json").ok()?;
        let mut data = String::new();
        file.read_to_string(&mut data).ok()?;
        serde_json::from_str(&data).ok()
    }

    pub fn save_to_file(&self) {
        let data = serde_json::to_string_pretty(self).unwrap();
        File::create("blockchain.json").unwrap().write_all(data.as_bytes()).unwrap();
    }

    pub fn add_transaction(&mut self, tx: Transaction) -> bool {
        if is_revstop_active(&tx.sender) {
            return false;
        }
        self.blocks.last_mut().unwrap().transactions.push(tx);
        self.save_to_file();
        true
    }

    pub fn mine_block(&mut self, miner_address: &str) {
        let last_block = self.blocks.last().unwrap();
        let new_block = Block::mine_from(last_block, miner_address.to_string());
        self.blocks.push(new_block);
        self.save_to_file();
    }

    pub fn get_balance(&self, address: &str) -> u64 {
        let mut balance = 0;
        for block in &self.blocks {
            for tx in &block.transactions {
                if tx.recipient == address {
                    balance += tx.amount;
                } else if tx.sender == address {
                    balance -= tx.amount;
                }
            }
        }
        balance
    }
}