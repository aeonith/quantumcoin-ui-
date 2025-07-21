use std::time::{SystemTime, UNIX_EPOCH};

/// Initial reward and halving
const INITIAL_REWARD: u64 = 25;
const HALVING_INTERVAL_BLOCKS: u64 = 210_000; // Bitcoin-like

/// Simple difficulty retarget every block (for demonstration)
pub fn calculate_block_reward(height: u64) -> u64 {
    INITIAL_REWARD >> (height / HALVING_INTERVAL_BLOCKS) as usize
}

pub fn get_timestamp() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}