use crate::{block::Block, transaction::Transaction};
use serde::{Deserialize, Serialize};
use std::{error::Error, fs, path::Path};

#[derive(Serialize, Deserialize, Debug)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    difficulty: u32,
}

impl Blockchain {
    pub fn new_with_genesis(miner_addr: &str) -> Self {
        let mut bc = Self { chain: vec![], pending_transactions: vec![], difficulty: 2 };
        bc.create_genesis_block(miner_addr);
        bc
    }

    pub fn chain_is_empty(&self) -> bool {
        self.chain.is_empty()
    }

    pub fn create_genesis_block(&mut self, miner_addr: &str) {
        let reward_tx = Transaction::new("GENESIS", miner_addr, 0.0);
        let genesis = Block::new(0, vec![reward_tx], "0".into());
        self.chain.push(genesis);
    }

    pub fn create_block_from_pending(&mut self, miner_addr: &str) -> Block {
        let reward_tx = Transaction::new("NETWORK", miner_addr, 1.0);
        self.pending_transactions.push(reward_tx);
        let new_block = Block::new(
            self.chain.len() as u64,
            self.pending_transactions.drain(..).collect(),
            self.chain.last().unwrap().hash.clone(),
        );
        self.chain.push(new_block.clone());
        new_block
    }

    pub fn mine_pending_transactions(&mut self, miner_addr: &str) -> bool {
        self.create_block_from_pending(miner_addr);
        true
    }

    /* ---------- Persistence helpers ---------- */

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        fs::write(path, serde_json::to_vec_pretty(self)?)?;
        Ok(())
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Option<Self> {
        fs::read(path).ok()
            .and_then(|bytes| serde_json::from_slice::<Self>(&bytes).ok())
    }
}