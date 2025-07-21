use serde::{Serialize, Deserialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use crate::block::Block;
use crate::transaction::Transaction;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub mempool: Vec<Transaction>,
    pub difficulty: u64,
    pub mining_reward: f64,
}

impl Blockchain {
    pub fn new() -> Self {
        let mut blockchain = Blockchain {
            chain: vec![],
            mempool: vec![],
            difficulty: 5, // match Bitcoin-style mining difficulty
            mining_reward: 50.0,
        };

        // Only create the genesis block if the chain is empty
        if blockchain.chain.is_empty() {
            let genesis_tx = Transaction {
                sender: "GENESIS".to_string(),
                recipient: "tNzCy5NT+GORGIA+JCVIGAJUIBM...QNSATLVTHNBWXMZA783YP/ALNCM2GEAO1TZ==".to_string(), // your pre-mined address
                amount: 1_250_000.0,
                signature: None,
            };

            let genesis_block = Block::new(0, vec![genesis_tx], "0".to_string(), blockchain.difficulty);
            blockchain.chain.push(genesis_block);
        }

        blockchain
    }

    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.mempool.push(transaction);
    }

    pub fn mine_pending_transactions(&mut self, miner_address: String) {
        if self.mempool.is_empty() {
            println!("⛏️  No transactions to mine.");
            return;
        }

        // Add mining reward
        let reward_tx = Transaction {
            sender: "Network".to_string(),
            recipient: miner_address,
            amount: self.mining_reward,
            signature: None,
        };
        self.mempool.push(reward_tx);

        let previous_hash = self.chain.last().unwrap().hash.clone();
        let new_block = Block::new(self.chain.len() as u64, self.mempool.clone(), previous_hash, self.difficulty);

        self.chain.push(new_block);
        self.mempool.clear();

        println!("✅ Block mined and added to chain!");
    }

    pub fn is_chain_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let current = &self.chain[i];
            let previous = &self.chain[i - 1];

            if current.hash != current.calculate_hash() {
                return false;
            }

            if current.previous_hash != previous.hash {
                return false;
            }
        }
        true
    }

    pub fn get_balance(&self, address: &str) -> f64 {
        let mut balance = 0.0;

        for block in &self.chain {
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

    pub fn save_to_file(&self, path: &str) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    pub fn load_from_file(path: &str) -> std::io::Result<Self> {
        let mut file = OpenOptions::new().read(true).open(path)?;
        let mut json = String::new();
        file.read_to_string(&mut json)?;
        let blockchain: Blockchain = serde_json::from_str(&json)?;
        Ok(blockchain)
    }
}