use serde::{Serialize, Deserialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::VecDeque;
use std::path::Path;
use base64::{encode, decode};
use pqcrypto_dilithium::dilithium2::{DetachedSignature, PublicKey};

#[derive(Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub sender: String,
    pub recipient: String,
    pub amount: f64,
    pub signature: Option<String>,
}

impl Transaction {
    pub fn new(sender: String, recipient: String, amount: f64, signature: Option<DetachedSignature>) -> Self {
        Transaction {
            sender,
            recipient,
            amount,
            signature: signature.map(|s| encode(s.as_bytes())),
        }
    }

    pub fn verify(&self) -> bool {
        if self.sender == "GENESIS" {
            return true;
        }

        if let Some(sig_b64) = &self.signature {
            if let Ok(sig_bytes) = decode(sig_b64) {
                if let Ok(pub_bytes) = decode(&self.sender) {
                    if let Ok(signature) = DetachedSignature::from_bytes(&sig_bytes) {
                        if let Ok(public_key) = PublicKey::from_bytes(&pub_bytes) {
                            let data = format!("{}{}{}", self.sender, self.recipient, self.amount);
                            return pqcrypto_dilithium::dilithium2::verify_detached(
                                &signature,
                                data.as_bytes(),
                                &public_key,
                            ).is_ok();
                        }
                    }
                }
            }
        }
        false
    }
}

#[derive(Serialize, Deserialize)]
pub struct Block {
    pub timestamp: u64,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub nonce: u64,
    pub hash: String,
}

impl Block {
    pub fn new(transactions: Vec<Transaction>, previous_hash: String) -> Self {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let nonce = 0;
        let mut block = Block {
            timestamp,
            transactions,
            previous_hash,
            nonce,
            hash: String::new(),
        };
        block.hash = block.calculate_hash();
        block
    }

    pub fn calculate_hash(&self) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(self.timestamp.to_string());
        hasher.update(&self.previous_hash);
        hasher.update(self.nonce.to_string());
        for tx in &self.transactions {
            hasher.update(format!("{}{}{}", tx.sender, tx.recipient, tx.amount));
        }
        let result = hasher.finalize();
        hex::encode(result)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_transactions: VecDeque<Transaction>,
}

impl Blockchain {
    pub fn load_or_new(genesis_recipient: String) -> Self {
        if Path::new("blockchain.json").exists() {
            let mut file = File::open("blockchain.json").unwrap();
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            serde_json::from_str(&contents).unwrap()
        } else {
            let mut blockchain = Blockchain {
                chain: vec![],
                pending_transactions: VecDeque::new(),
            };

            let genesis_tx = Transaction::new(
                "GENESIS".to_string(),
                genesis_recipient,
                1_250_000.0,
                None,
            );
            let genesis_block = Block::new(vec![genesis_tx], String::from("0"));
            blockchain.chain.push(genesis_block);
            blockchain.save_to_disk();
            blockchain
        }
    }

    pub fn save_to_disk(&self) {
        let json = serde_json::to_string_pretty(&self).unwrap();
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open("blockchain.json")
            .unwrap();
        file.write_all(json.as_bytes()).unwrap();
    }

    pub fn add_transaction(&mut self, tx: Transaction) -> bool {
        if tx.verify() {
            self.pending_transactions.push_back(tx);
            true
        } else {
            false
        }
    }

    pub fn mine_pending(&mut self, miner_address: String) {
        let reward_tx = Transaction::new(
            "GENESIS".to_string(),
            miner_address,
            50.0,
            None,
        );
        self.pending_transactions.push_back(reward_tx);

        let txs: Vec<_> = self.pending_transactions.drain(..).collect();
        let last_hash = self.chain.last().map(|b| b.hash.clone()).unwrap_or_else(|| "0".to_string());
        let block = Block::new(txs, last_hash);
        self.chain.push(block);
        self.save_to_disk();
    }

    pub fn get_balance(&self, address: &str) -> f64 {
        let mut balance = 0.0;
        for block in &self.chain {
            for tx in &block.transactions {
                if tx.recipient == address {
                    balance += tx.amount;
                }
                if tx.sender == address {
                    balance -= tx.amount;
                }
            }
        }
        balance
    }

    pub fn get_last_n_transactions(&self, address: &str, n: usize) -> Vec<Transaction> {
        let mut results = vec![];
        for block in self.chain.iter().rev() {
            for tx in block.transactions.iter().rev() {
                if tx.sender == address || tx.recipient == address {
                    results.push(tx.clone());
                    if results.len() >= n {
                        return results;
                    }
                }
            }
        }
        results
    }
}