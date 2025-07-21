use std::time::{SystemTime, UNIX_EPOCH};

/// Initial mining reward in QTC
const INITIAL_REWARD: f64 = 50.0;

/// Max total supply of QuantumCoin
pub const MAX_SUPPLY: f64 = 22_000_000.0;

/// Approx. time between blocks (Bitcoin-style: 10 minutes)
const BLOCK_TIME_SECONDS: u64 = 600;

/// Number of blocks per halving (~2 years worth of blocks at 10 min/block)
const HALVING_INTERVAL_BLOCKS: u64 = 105120; // 365*24*6 = 105120 blocks per 2 years

/// Minimum possible block reward (prevents rewards from going below dust value)
const MIN_REWARD_THRESHOLD: f64 = 0.0001;

/// Returns the current UNIX timestamp in seconds
pub fn get_current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Calculates the mining reward based on current block height
pub fn calculate_block_reward(block_height: u64) -> f64 {
    let halvings = block_height / HALVING_INTERVAL_BLOCKS;
    let reward = INITIAL_REWARD / 2f64.powi(halvings as i32);

    if reward < MIN_REWARD_THRESHOLD {
        0.0
    } else {
        reward
    }
}

/// Calculates difficulty based on current block height and previous timestamp
/// This is a simplified dynamic difficulty system
pub fn calculate_difficulty(previous_timestamp: u64, block_height: u64) -> u64 {
    let current_timestamp = get_current_timestamp();
    let time_elapsed = current_timestamp - previous_timestamp;

    let expected_time = BLOCK_TIME_SECONDS * block_height;
    let base_difficulty = 5;

    if time_elapsed < expected_time {
        base_difficulty + (block_height / 1000) // increase with time
    } else {
        base_difficulty // fallback base level
    }
}