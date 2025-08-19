use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// QuantumCoin Economics Engine
/// 
/// Handles emission curve, fee estimation, and economic policy enforcement
pub struct EconomicsEngine {
    /// Chain specification parameters
    max_supply: u64,
    initial_reward: u64,
    halving_interval: u64,
    target_block_time: u64,
    
    /// Fee estimation data
    recent_blocks: VecDeque<BlockFeeData>,
    confirmation_targets: Vec<u32>,
    
    /// Network metrics
    total_supply: u64,
    current_height: u64,
}

/// Block fee data for estimation
#[derive(Debug, Clone)]
struct BlockFeeData {
    height: u64,
    timestamp: u64,
    transactions: Vec<TransactionFeeData>,
    total_fees: u64,
    block_size: u64,
}

/// Transaction fee data
#[derive(Debug, Clone)]
struct TransactionFeeData {
    fee_per_byte: f64,
    confirmation_time: u64, // blocks until confirmation
    size: u64,
    fee: u64,
}

/// Fee estimation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeEstimate {
    pub target_blocks: u32,
    pub fee_per_byte: f64,
    pub total_fee_estimate: u64,
    pub confidence: f64, // 0.0 - 1.0
}

/// Economic statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicStats {
    pub total_supply: u64,
    pub circulating_supply: u64,
    pub max_supply: u64,
    pub inflation_rate: f64,
    pub current_reward: u64,
    pub next_halving_height: u64,
    pub blocks_until_halving: u64,
    pub emission_percentage: f64,
    pub avg_block_time: f64,
    pub difficulty_adjustment_ratio: f64,
}

/// Supply curve point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplyPoint {
    pub height: u64,
    pub supply: u64,
    pub reward: u64,
    pub inflation_rate: f64,
}

impl EconomicsEngine {
    /// Create new economics engine
    pub fn new() -> Self {
        // QuantumCoin parameters from chain specification
        Self {
            max_supply: 22_000_000_00000000, // 22M QTC with 8 decimals
            initial_reward: 50_00000000,     // 50 QTC initial reward
            halving_interval: 210_000,       // Every 210,000 blocks (~4 years)
            target_block_time: 600,          // 10 minutes in seconds
            
            recent_blocks: VecDeque::with_capacity(2016), // Keep 2 weeks of blocks
            confirmation_targets: vec![1, 3, 6, 12, 24, 144], // blocks
            
            total_supply: 0,
            current_height: 0,
        }
    }
    
    /// Calculate block reward at given height
    pub fn calculate_block_reward(&self, height: u64) -> u64 {
        let halvings = height / self.halving_interval;
        
        if halvings >= 64 {
            return 0; // All rewards distributed after 64 halvings
        }
        
        self.initial_reward >> halvings
    }
    
    /// Calculate total supply at given height
    pub fn calculate_supply_at_height(&self, height: u64) -> u64 {
        let mut supply = 0u64;
        let mut current_height = 0u64;
        
        while current_height < height {
            let reward = self.calculate_block_reward(current_height);
            if reward == 0 {
                break;
            }
            
            let next_halving = ((current_height / self.halving_interval) + 1) * self.halving_interval;
            let blocks_at_this_reward = std::cmp::min(next_halving - current_height, height - current_height);
            
            supply = supply.saturating_add(reward.saturating_mul(blocks_at_this_reward));
            current_height += blocks_at_this_reward;
        }
        
        supply
    }
    
    /// Generate complete emission curve
    pub fn generate_emission_curve(&self) -> Vec<SupplyPoint> {
        let mut curve = Vec::new();
        let mut current_supply = 0u64;
        
        for halving_epoch in 0..33 { // 33 halvings cover full emission
            let height_start = halving_epoch * self.halving_interval;
            let height_end = (halving_epoch + 1) * self.halving_interval;
            let reward = self.calculate_block_reward(height_start);
            
            if reward == 0 {
                break;
            }
            
            let blocks_in_epoch = self.halving_interval;
            let epoch_emission = reward * blocks_in_epoch;
            
            // Add point at start of epoch
            curve.push(SupplyPoint {
                height: height_start,
                supply: current_supply,
                reward,
                inflation_rate: if current_supply > 0 {
                    (epoch_emission as f64 / current_supply as f64) * 100.0
                } else { 
                    100.0 // First epoch is 100% inflation from 0
                },
            });
            
            current_supply = current_supply.saturating_add(epoch_emission);
            
            // Add point at end of epoch
            curve.push(SupplyPoint {
                height: height_end - 1,
                supply: current_supply,
                reward,
                inflation_rate: (epoch_emission as f64 / current_supply as f64) * 100.0,
            });
        }
        
        curve
    }
    
    /// Update with new block data
    pub fn update_with_block(
        &mut self, 
        height: u64, 
        timestamp: u64, 
        transactions: Vec<(u64, u64)>, // (fee, size) pairs
        total_fees: u64,
        block_size: u64,
    ) {
        self.current_height = height;
        self.total_supply = self.calculate_supply_at_height(height);
        
        let fee_data: Vec<TransactionFeeData> = transactions
            .into_iter()
            .map(|(fee, size)| TransactionFeeData {
                fee_per_byte: if size > 0 { fee as f64 / size as f64 } else { 0.0 },
                confirmation_time: 1, // Confirmed in this block
                size,
                fee,
            })
            .collect();
        
        let block_fee_data = BlockFeeData {
            height,
            timestamp,
            transactions: fee_data,
            total_fees,
            block_size,
        };
        
        self.recent_blocks.push_back(block_fee_data);
        
        // Keep only recent blocks
        while self.recent_blocks.len() > 2016 {
            self.recent_blocks.pop_front();
        }
    }
    
    /// Estimate fee for target confirmation time
    pub fn estimate_fee(&self, target_blocks: u32, transaction_size: u64) -> FeeEstimate {
        if self.recent_blocks.is_empty() {
            return FeeEstimate {
                target_blocks,
                fee_per_byte: 0.001, // Default minimum fee
                total_fee_estimate: transaction_size.saturating_mul(100), // 0.001 QTC per byte
                confidence: 0.1,
            };
        }
        
        let mut fees: Vec<f64> = Vec::new();
        
        // Collect fees from recent blocks
        let blocks_to_analyze = std::cmp::min(target_blocks as usize * 10, self.recent_blocks.len());
        for block in self.recent_blocks.iter().rev().take(blocks_to_analyze) {
            for tx in &block.transactions {
                if tx.confirmation_time <= target_blocks as u64 {
                    fees.push(tx.fee_per_byte);
                }
            }
        }
        
        if fees.is_empty() {
            return FeeEstimate {
                target_blocks,
                fee_per_byte: 0.001,
                total_fee_estimate: transaction_size.saturating_mul(100),
                confidence: 0.1,
            };
        }
        
        // Sort fees for percentile calculation
        fees.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        // Use different percentiles based on target confirmation time
        let percentile = match target_blocks {
            1 => 0.9,      // 90th percentile for next block
            2..=3 => 0.75, // 75th percentile for 2-3 blocks
            4..=6 => 0.5,  // 50th percentile for 4-6 blocks
            _ => 0.25,     // 25th percentile for longer confirmations
        };
        
        let index = ((fees.len() - 1) as f64 * percentile) as usize;
        let fee_per_byte = fees[index].max(0.0001); // Minimum 0.0001 QTC/byte
        
        let confidence = if fees.len() >= 100 { 0.95 } 
                        else if fees.len() >= 50 { 0.85 }
                        else if fees.len() >= 10 { 0.7 }
                        else { 0.5 };
        
        FeeEstimate {
            target_blocks,
            fee_per_byte,
            total_fee_estimate: (fee_per_byte * transaction_size as f64) as u64,
            confidence,
        }
    }
    
    /// Get economic statistics
    pub fn get_statistics(&self) -> EconomicStats {
        let current_reward = self.calculate_block_reward(self.current_height);
        let current_halving_epoch = self.current_height / self.halving_interval;
        let next_halving_height = (current_halving_epoch + 1) * self.halving_interval;
        let blocks_until_halving = next_halving_height.saturating_sub(self.current_height);
        
        let circulating_supply = self.total_supply;
        let emission_percentage = if self.max_supply > 0 {
            (circulating_supply as f64 / self.max_supply as f64) * 100.0
        } else {
            0.0
        };
        
        // Calculate annual inflation rate
        let blocks_per_year = 365 * 24 * 60 * 60 / self.target_block_time;
        let yearly_emission = current_reward * blocks_per_year;
        let inflation_rate = if circulating_supply > 0 {
            (yearly_emission as f64 / circulating_supply as f64) * 100.0
        } else {
            0.0
        };
        
        // Calculate average block time from recent blocks
        let avg_block_time = if self.recent_blocks.len() >= 2 {
            let first_timestamp = self.recent_blocks.front().unwrap().timestamp;
            let last_timestamp = self.recent_blocks.back().unwrap().timestamp;
            let time_diff = last_timestamp - first_timestamp;
            let block_count = self.recent_blocks.len() as u64;
            
            if block_count > 1 && time_diff > 0 {
                time_diff as f64 / (block_count - 1) as f64
            } else {
                self.target_block_time as f64
            }
        } else {
            self.target_block_time as f64
        };
        
        let difficulty_adjustment_ratio = avg_block_time / self.target_block_time as f64;
        
        EconomicStats {
            total_supply: circulating_supply,
            circulating_supply,
            max_supply: self.max_supply,
            inflation_rate,
            current_reward,
            next_halving_height,
            blocks_until_halving,
            emission_percentage,
            avg_block_time,
            difficulty_adjustment_ratio,
        }
    }
    
    /// Validate economic rules for a block
    pub fn validate_block_economics(
        &self, 
        height: u64, 
        coinbase_amount: u64, 
        total_fees: u64
    ) -> Result<()> {
        let expected_reward = self.calculate_block_reward(height);
        let expected_coinbase = expected_reward + total_fees;
        
        if coinbase_amount != expected_coinbase {
            return Err(anyhow!(
                "Invalid coinbase amount: expected {}, got {}",
                expected_coinbase,
                coinbase_amount
            ));
        }
        
        // Check supply cap
        let new_supply = self.calculate_supply_at_height(height + 1);
        if new_supply > self.max_supply {
            return Err(anyhow!(
                "Supply cap exceeded: {} > {}",
                new_supply,
                self.max_supply
            ));
        }
        
        Ok(())
    }
    
    /// Get fee estimates for all confirmation targets
    pub fn get_all_fee_estimates(&self, transaction_size: u64) -> Vec<FeeEstimate> {
        self.confirmation_targets
            .iter()
            .map(|&target| self.estimate_fee(target, transaction_size))
            .collect()
    }
    
    /// Get mempool fee statistics
    pub fn analyze_mempool(&self, mempool_fees: Vec<f64>) -> MempoolAnalysis {
        if mempool_fees.is_empty() {
            return MempoolAnalysis {
                transaction_count: 0,
                min_fee: 0.0,
                max_fee: 0.0,
                avg_fee: 0.0,
                median_fee: 0.0,
                fee_percentiles: vec![],
            };
        }
        
        let mut sorted_fees = mempool_fees.clone();
        sorted_fees.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let min_fee = sorted_fees[0];
        let max_fee = sorted_fees[sorted_fees.len() - 1];
        let avg_fee = sorted_fees.iter().sum::<f64>() / sorted_fees.len() as f64;
        let median_fee = sorted_fees[sorted_fees.len() / 2];
        
        let percentiles = vec![10, 25, 50, 75, 90, 95, 99];
        let fee_percentiles: Vec<(u8, f64)> = percentiles
            .into_iter()
            .map(|p| {
                let index = ((sorted_fees.len() - 1) as f64 * (p as f64 / 100.0)) as usize;
                (p, sorted_fees[index])
            })
            .collect();
        
        MempoolAnalysis {
            transaction_count: mempool_fees.len(),
            min_fee,
            max_fee,
            avg_fee,
            median_fee,
            fee_percentiles,
        }
    }
}

/// Mempool fee analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MempoolAnalysis {
    pub transaction_count: usize,
    pub min_fee: f64,
    pub max_fee: f64,
    pub avg_fee: f64,
    pub median_fee: f64,
    pub fee_percentiles: Vec<(u8, f64)>, // (percentile, fee_per_byte)
}

impl Default for EconomicsEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_block_reward_calculation() {
        let engine = EconomicsEngine::new();
        
        // Test initial reward
        assert_eq!(engine.calculate_block_reward(0), 50_00000000);
        assert_eq!(engine.calculate_block_reward(100_000), 50_00000000);
        
        // Test first halving
        assert_eq!(engine.calculate_block_reward(210_000), 25_00000000);
        assert_eq!(engine.calculate_block_reward(300_000), 25_00000000);
        
        // Test second halving
        assert_eq!(engine.calculate_block_reward(420_000), 12_50000000);
        
        // Test eventual zero reward
        assert_eq!(engine.calculate_block_reward(64 * 210_000), 0);
    }
    
    #[test]
    fn test_supply_calculation() {
        let engine = EconomicsEngine::new();
        
        // Test genesis supply
        assert_eq!(engine.calculate_supply_at_height(0), 0);
        
        // Test after first block
        assert_eq!(engine.calculate_supply_at_height(1), 50_00000000);
        
        // Test after first halving interval
        let supply_at_first_halving = engine.calculate_supply_at_height(210_000);
        let expected = 50_00000000 * 210_000; // 10.5M QTC
        assert_eq!(supply_at_first_halving, expected);
    }
    
    #[test]
    fn test_emission_curve_generation() {
        let engine = EconomicsEngine::new();
        let curve = engine.generate_emission_curve();
        
        assert!(!curve.is_empty());
        assert_eq!(curve[0].height, 0);
        assert_eq!(curve[0].supply, 0);
        assert_eq!(curve[0].reward, 50_00000000);
        
        // Check that supply increases monotonically
        for window in curve.windows(2) {
            assert!(window[1].supply >= window[0].supply);
        }
    }
    
    #[test]
    fn test_fee_estimation() {
        let mut engine = EconomicsEngine::new();
        
        // Add some block data
        engine.update_with_block(
            1, 
            1000, 
            vec![(100_000, 250), (200_000, 500)], // (fee, size)
            300_000,
            1000,
        );
        
        let estimate = engine.estimate_fee(1, 250);
        assert!(estimate.fee_per_byte > 0.0);
        assert!(estimate.total_fee_estimate > 0);
        assert!(estimate.confidence > 0.0);
    }
    
    #[test]
    fn test_economic_validation() {
        let engine = EconomicsEngine::new();
        
        // Valid block economics
        assert!(engine.validate_block_economics(0, 50_00000000, 0).is_ok());
        assert!(engine.validate_block_economics(1, 50_01000000, 1000000).is_ok());
        
        // Invalid coinbase amount
        assert!(engine.validate_block_economics(0, 60_00000000, 0).is_err());
        
        // Invalid fee calculation
        assert!(engine.validate_block_economics(1, 50_00000000, 1000000).is_err());
    }
    
    #[test]
    fn test_mempool_analysis() {
        let engine = EconomicsEngine::new();
        let fees = vec![0.001, 0.002, 0.005, 0.010, 0.020, 0.050];
        
        let analysis = engine.analyze_mempool(fees);
        
        assert_eq!(analysis.transaction_count, 6);
        assert_eq!(analysis.min_fee, 0.001);
        assert_eq!(analysis.max_fee, 0.050);
        assert!(analysis.avg_fee > 0.0);
        assert!(analysis.median_fee > 0.0);
        assert!(!analysis.fee_percentiles.is_empty());
    }
}
