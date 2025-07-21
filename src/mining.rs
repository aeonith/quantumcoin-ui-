use std::time::{SystemTime, UNIX_EPOCH};

const INITIAL_REWARD: u64 = 50; // Initial block reward
const HALVING_INTERVAL: u64 = 1_051_200; // ~2 years assuming 60s blocks
const MAX_SUPPLY: u64 = 22_000_000; // Total QuantumCoin supply cap
const GENESIS_REWARD: u64 = 1_250_000; // Genesis block allocation

pub fn get_block_reward(block_height: u64, current_total_supply: u64) -> u64 {
    // No reward if max supply reached
    if current_total_supply >= MAX_SUPPLY {
        return 0;
    }

    // Number of halvings already occurred
    let halvings = block_height / HALVING_INTERVAL;
    let reward = INITIAL_REWARD >> halvings;

    // Prevent shifting below 1 satoshi equivalent
    if reward == 0 {
        return 0;
    }

    // Cap the reward if close to total max
    let remaining = MAX_SUPPLY - current_total_supply;
    if reward > remaining {
        return remaining;
    }

    reward
}

pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}