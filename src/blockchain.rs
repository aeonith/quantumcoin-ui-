use serde::{Serialize, Deserialize};
use std::{fs, path::Path};
use crate::transaction::Transaction;

/// A single block in the chain.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Block {
    pub index:        u64,
    pub timestamp:    u128,
    pub transactions: Vec<Transaction>,
    pub previous_hash:String,
    pub hash:         String,
    pub nonce:        u64,
}

/// The full blockchain.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Blockchain {
    pub chain:              Vec<Block>,
    pub difficulty:         u32,        // number of leading zeros
    pub pending_transactions: Vec<Transaction>,
    pub mining_reward:      u64,        // reward per block
}

impl Blockchain {
    /// Load or initialize with genesis block.
    pub fn load_or_create() -> Self {
        let path = "blockchain.json";
        if Path::new(path).exists() {
            let data = fs::read_to_string(path).unwrap();
            serde_json::from_str(&data).unwrap()
        } else {
            let mut bc = Blockchain {
                chain: vec![], difficulty: 4,
                pending_transactions: vec![],
                mining_reward: 50,
            };
            bc.create_genesis();
            bc.save();
            bc
        }
    }

    pub fn save(&self) {
        fs::write("blockchain.json", serde_json::to_string_pretty(self).unwrap()).unwrap();
    }

    fn create_genesis(&mut self) {
        let genesis = Block {
            index: 0,
            timestamp: now(),
            transactions: vec![],
            previous_hash: "0".into(),
            hash: "genesis_hash".into(),
            nonce: 0,
        };
        self.chain.push(genesis);
    }

    pub fn latest_block(&self) -> &Block {
        self.chain.last().unwrap()
    }

    /// Proof‐of‐Work mining: find a nonce producing a hash with `difficulty` zeros.
    pub fn mine_pending(&mut self, reward_address: &str) {
        // create new block with pending txs + reward
        let mut block = Block {
            index: self.chain.len() as u64,
            timestamp: now(),
            transactions: std::mem::take(&mut self.pending_transactions),
            previous_hash: self.latest_block().hash.clone(),
            hash: String::new(),
            nonce: 0,
        };

        // mining loop
        loop {
            let data = format!("{:?}{:?}{:?}{:?}", block.index, block.timestamp, block.nonce, block.transactions);
            let hash = sha2::Sha256::digest(data.as_bytes());
            let hash_hex = hex::encode(hash);
            if hash_hex.starts_with(&"0".repeat(self.difficulty as usize)) {
                block.hash = hash_hex;
                break;
            }
            block.nonce += 1;
        }

        // reward transaction
        let reward_tx = Transaction {
            sender: "SYSTEM".into(),
            recipient: reward_address.into(),
            amount: self.mining_reward,
            timestamp: now(),
        };
        block.transactions.push(reward_tx);

        self.chain.push(block);
        self.save();
    }

    pub fn add_transaction(&mut self, tx: Transaction) {
        self.pending_transactions.push(tx);
    }

    pub fn get_balance(&self, address: &str) -> u64 {
        let mut balance: i128 = 0;
        for block in &self.chain {
            for tx in &block.transactions {
                if tx.recipient == address { balance += tx.amount as i128 }
                if tx.sender    == address { balance -= tx.amount as i128 }
            }
        }
        balance.max(0) as u64
    }
}

fn now() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()
}