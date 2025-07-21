use sha2::{Digest, Sha256};
use chrono::Utc;

pub const INITIAL_REWARD: f64 = 50.0;
pub const HALVING_INTERVAL: u64 = 1_051_200; // approx every 2 years @ 1 block per min
pub const DIFFICULTY_PREFIX: &str = "0000"; // adjust for hardness

pub fn calculate_reward(block_index: u64) -> f64 {
    let halvings = block_index / HALVING_INTERVAL;
    let reward = INITIAL_REWARD / 2f64.powi(halvings as i32);
    if reward < 0.00000001 { 0.0 } else { reward }
}

pub fn mine_block(index: u64, previous_hash: &str, transactions_data: &str, timestamp: i64) -> (String, u64) {
    let mut nonce = 0;
    loop {
        let block_contents = format!("{}{}{}{}{}", index, previous_hash, transactions_data, timestamp, nonce);
        let hash = Sha256::digest(block_contents.as_bytes());
        let hash_hex = format!("{:x}", hash);

        if hash_hex.starts_with(DIFFICULTY_PREFIX) {
            return (hash_hex, nonce);
        }

        nonce += 1;
    }
}