use std::time::{Duration, Instant};

pub struct MiningInfo {
    pub last_mined: Instant,
    pub mining_difficulty: u64,
    pub mining_reward: f64,
    pub halving_interval_secs: u64,
    pub reward_halvings: u32,
}

impl MiningInfo {
    pub fn new() -> Self {
        Self {
            last_mined: Instant::now(),
            mining_difficulty: 5,          // Difficulty = number of leading zeros required
            mining_reward: 6.25,           // Starting mining reward (similar to Bitcoin post-halvings)
            halving_interval_secs: 60 * 60 * 24 * 365 * 2, // Halves every 2 years
            reward_halvings: 0,
        }
    }

    pub fn update_reward_if_needed(&mut self) {
        let time_since_last_halving = self.last_mined.elapsed().as_secs();
        if time_since_last_halving > self.halving_interval_secs {
            self.reward_halvings += 1;
            self.mining_reward /= 2.0;
            self.last_mined = Instant::now();
            println!(
                "⛏️ Mining reward halved. New reward: {} QTC",
                self.mining_reward
            );
        }
    }

    pub fn get_current_reward(&self) -> f64 {
        self.mining_reward
    }

    pub fn get_current_difficulty(&self) -> u64 {
        self.mining_difficulty
    }

    pub fn increase_difficulty(&mut self) {
        self.mining_difficulty += 1;
    }

    pub fn show_status(&self) {
        println!(
            "⛏️ Current mining reward: {} QTC | Difficulty: {} | Halvings: {}",
            self.mining_reward, self.mining_difficulty, self.reward_halvings
        );
    }
}