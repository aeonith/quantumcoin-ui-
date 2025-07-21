// src/mining.rs

use std::time::{SystemTime, UNIX_EPOCH};

pub const INITIAL_REWARD: u64 = 50;
pub const HALVING_INTERVAL: u64 = 1_051_200; // Approx 2 years at 1 block per minute
pub const MAX_SUPPLY: u64 = 22_000_000;
pub const GENESIS_REWARD: u64 = 1_250_000;

pub fn get_block_reward(block_height: u64, current_supply: u64) -> u64 {
    if current_supply >= MAX_SUPPLY {
        return 0;
    }

    let halvings = block_height / HALVING_INTERVAL;
    let reward = INITIAL_REWARD >> (halvings as u32); // Reward halves each interval

    // Ensure we don't exceed total supply
    let remaining = MAX_SUPPLY - current_supply;
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