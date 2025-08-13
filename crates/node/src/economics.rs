//! Economic model implementation for QuantumCoin
//! 
//! This module implements the issuance schedule and supply calculations
//! based on the canonical parameters defined in the configuration.

use crate::config::{EconomicsConfig, SharedConfig};
use serde::{Deserialize, Serialize};

/// Block height type
pub type BlockHeight = u64;

/// Economics engine that calculates issuance and supply
#[derive(Debug, Clone)]
pub struct Economics {
    config: EconomicsConfig,
}

impl Economics {
    /// Create new economics engine from configuration
    pub fn new(config: EconomicsConfig) -> Self {
        Economics { config }
    }
    
    /// Create from shared configuration
    pub fn from_shared_config(config: &SharedConfig) -> Self {
        Economics::new(config.economics().clone())
    }
    
    /// Calculate the number of blocks per halving period
    pub fn blocks_per_halving(&self) -> u64 {
        let seconds_per_year = 365 * 24 * 60 * 60;
        let seconds_per_halving = self.config.halving_period_years as u64 * seconds_per_year;
        seconds_per_halving / self.config.block_time_target_sec as u64
    }
    
    /// Calculate total number of halvings over the entire duration
    pub fn total_halvings(&self) -> u32 {
        self.config.halving_duration_years / self.config.halving_period_years
    }
    
    /// Calculate initial block reward (after genesis/dev allocations)
    pub fn initial_block_reward(&self) -> u64 {
        let allocated = self.config.genesis_premine_qtc + self.config.dev_fund_qtc;
        let remaining = self.config.total_supply - allocated;
        
        // Calculate total blocks over all halvings
        let total_halvings = self.total_halvings();
        let blocks_per_halving = self.blocks_per_halving();
        
        // Sum of geometric series: a * (1 - r^n) / (1 - r)
        // where a = initial_reward, r = 0.5, n = number_of_halvings
        // We need to solve for 'a' given the total supply
        
        let geometric_sum_factor = self.calculate_geometric_sum_factor(total_halvings);
        let total_blocks = blocks_per_halving * total_halvings as u64;
        
        // Distribute remaining supply over all blocks with halving
        let base_reward = remaining / total_blocks;
        
        // Adjust for geometric series to get initial reward
        (base_reward * 2) / geometric_sum_factor as u64
    }
    
    /// Calculate the sum factor for geometric series
    fn calculate_geometric_sum_factor(&self, halvings: u32) -> f64 {
        let mut factor = 0.0;
        for i in 0..halvings {
            factor += 0.5_f64.powi(i as i32);
        }
        factor
    }
    
    /// Calculate block reward for a given height
    pub fn block_reward(&self, height: BlockHeight) -> u64 {
        if height == 0 {
            // Genesis block gets premine
            return self.config.genesis_premine_qtc;
        }
        
        let halving_period = self.blocks_per_halving();
        let halvings = (height - 1) / halving_period;
        let total_halvings = self.total_halvings();
        
        if halvings >= total_halvings as u64 {
            // No more rewards after all halvings complete
            return 0;
        }
        
        let initial_reward = self.initial_block_reward();
        initial_reward / (2_u64.pow(halvings as u32))
    }
    
    /// Calculate cumulative issuance up to a given height
    pub fn cumulative_issuance(&self, height: BlockHeight) -> u64 {
        if height == 0 {
            return 0;
        }
        
        let mut total = self.config.genesis_premine_qtc; // Genesis block
        let halving_period = self.blocks_per_halving();
        let initial_reward = self.initial_block_reward();
        
        // Calculate rewards for each halving period
        let mut current_height = 1;
        let mut halving = 0;
        
        while current_height <= height {
            let period_end = std::cmp::min(
                height + 1,
                (halving + 1) * halving_period + 1
            );
            let blocks_in_period = period_end - current_height;
            
            if halving < self.total_halvings() as u64 {
                let reward_per_block = initial_reward / (2_u64.pow(halving as u32));
                total += blocks_in_period * reward_per_block;
            }
            
            current_height = period_end;
            halving += 1;
        }
        
        total
    }
    
    /// Get maximum possible supply
    pub fn max_supply(&self) -> u64 {
        self.config.total_supply
    }
    
    /// Calculate remaining supply to be issued
    pub fn remaining_supply(&self, height: BlockHeight) -> u64 {
        self.max_supply() - self.cumulative_issuance(height)
    }
    
    /// Check if all coins have been issued
    pub fn issuance_complete(&self, height: BlockHeight) -> bool {
        self.remaining_supply(height) == 0
    }
    
    /// Get next halving height
    pub fn next_halving_height(&self, current_height: BlockHeight) -> Option<BlockHeight> {
        let halving_period = self.blocks_per_halving();
        let current_halving = (current_height.saturating_sub(1)) / halving_period;
        let next_halving = current_halving + 1;
        
        if next_halving >= self.total_halvings() as u64 {
            None // No more halvings
        } else {
            Some(next_halving * halving_period + 1)
        }
    }
}

/// Issuance schedule information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuanceSchedule {
    /// Current block height
    pub height: BlockHeight,
    
    /// Current block reward
    pub current_reward: u64,
    
    /// Cumulative issued coins
    pub total_issued: u64,
    
    /// Remaining coins to issue
    pub remaining: u64,
    
    /// Next halving height (if any)
    pub next_halving: Option<BlockHeight>,
    
    /// Blocks until next halving
    pub blocks_to_halving: Option<u64>,
    
    /// Current halving period
    pub current_period: u64,
}

impl Economics {
    /// Get complete issuance schedule information for a height
    pub fn issuance_schedule(&self, height: BlockHeight) -> IssuanceSchedule {
        let current_reward = self.block_reward(height);
        let total_issued = self.cumulative_issuance(height);
        let remaining = self.remaining_supply(height);
        let next_halving = self.next_halving_height(height);
        
        let blocks_to_halving = next_halving.map(|h| h.saturating_sub(height));
        let halving_period = self.blocks_per_halving();
        let current_period = if height == 0 { 0 } else { (height - 1) / halving_period };
        
        IssuanceSchedule {
            height,
            current_reward,
            total_issued,
            remaining,
            next_halving,
            blocks_to_halving,
            current_period,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ChainConfig;
    
    fn test_economics() -> Economics {
        Economics::new(ChainConfig::default().economics)
    }
    
    #[test]
    fn test_supply_conservation() {
        let economics = test_economics();
        
        // Test that cumulative issuance never exceeds max supply
        for height in [0, 1000, 10000, 100000, 1000000] {
            let issued = economics.cumulative_issuance(height);
            assert!(
                issued <= economics.max_supply(),
                "Issued {} exceeds max supply {} at height {}",
                issued,
                economics.max_supply(),
                height
            );
        }
    }
    
    #[test]
    fn test_issuance_monotonic() {
        let economics = test_economics();
        
        // Test that cumulative issuance is monotonically increasing
        let mut prev_issued = 0;
        for height in 0..1000 {
            let issued = economics.cumulative_issuance(height);
            assert!(
                issued >= prev_issued,
                "Issuance decreased from {} to {} at height {}",
                prev_issued,
                issued,
                height
            );
            prev_issued = issued;
        }
    }
    
    #[test]
    fn test_halving_schedule() {
        let economics = test_economics();
        let halving_period = economics.blocks_per_halving();
        
        // Test that rewards halve at the right intervals
        let reward_0 = economics.block_reward(1);
        let reward_1 = economics.block_reward(halving_period + 1);
        let reward_2 = economics.block_reward(2 * halving_period + 1);
        
        assert_eq!(reward_1 * 2, reward_0, "First halving should halve reward");
        assert_eq!(reward_2 * 2, reward_1, "Second halving should halve reward");
    }
    
    #[test]
    fn test_final_supply() {
        let economics = test_economics();
        
        // Calculate issuance at a very high block height
        let final_height = 10_000_000; // Should be well beyond all halvings
        let final_issued = economics.cumulative_issuance(final_height);
        
        // Should equal max supply (within rounding errors)
        let max_supply = economics.max_supply();
        assert!(
            final_issued <= max_supply,
            "Final issued {} exceeds max {}",
            final_issued,
            max_supply
        );
        
        // Should be very close to max supply
        let diff = max_supply - final_issued;
        assert!(
            diff < 1000, // Allow small difference due to discrete blocks
            "Final supply differs by {} from max",
            diff
        );
    }
    
    #[test]
    fn test_genesis_allocation() {
        let economics = test_economics();
        
        // Genesis block should have premine
        assert_eq!(
            economics.block_reward(0),
            economics.config.genesis_premine_qtc
        );
        
        // Regular blocks should have calculated reward
        let regular_reward = economics.block_reward(1);
        assert!(regular_reward > 0);
        assert!(regular_reward < economics.config.genesis_premine_qtc);
    }
    
    #[test]
    fn test_issuance_schedule_info() {
        let economics = test_economics();
        let schedule = economics.issuance_schedule(1000);
        
        assert_eq!(schedule.height, 1000);
        assert!(schedule.current_reward > 0);
        assert!(schedule.total_issued > 0);
        assert!(schedule.remaining > 0);
        assert_eq!(schedule.total_issued + schedule.remaining, economics.max_supply());
    }
}
