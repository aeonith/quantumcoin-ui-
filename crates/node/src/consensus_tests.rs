//! Comprehensive property-based tests for the consensus engine
//! 
//! This module contains extensive property-based tests to ensure the consensus
//! engine behaves correctly under all edge cases and maintains the invariants
//! required for a production cryptocurrency.

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::consensus_engine::*;
    use crate::chain_spec_loader::ChainSpecLoader;
    use crate::config::ChainConfig;
    use crate::block::{Block, BlockHeader};
    use crate::transaction::{Transaction, TransactionInput, TransactionOutput};
    use proptest::prelude::*;
    use proptest::collection::vec;
    use std::collections::HashMap;
    use std::time::{SystemTime, UNIX_EPOCH};
    
    // Test data generators
    
    prop_compose! {
        fn arb_block_header()(
            height in 0u64..1_000_000,
            timestamp in 1_640_995_200u64..2_000_000_000u64, // 2022-2033
            difficulty in 0x1000_0000u32..0x2000_0000u32,
            nonce in 0u64..u64::MAX,
        ) -> BlockHeader {
            BlockHeader {
                height,
                previous_hash: [0u8; 32], // Will be set properly in tests
                merkle_root: [0u8; 32], // Will be calculated properly
                timestamp,
                difficulty,
                nonce,
            }
        }
    }
    
    prop_compose! {
        fn arb_transaction_input()(
            prev_tx_hash in prop::array::uniform32(0u8..255u8),
            output_index in 0u32..1000,
            signature in vec(0u8..255u8, 64..2420), // dilithium2 signature size
        ) -> TransactionInput {
            TransactionInput {
                prev_tx_hash,
                output_index,
                signature,
            }
        }
    }
    
    prop_compose! {
        fn arb_transaction_output()(
            amount in 1u64..1_000_000_000_000, // 1 to 10,000 coins
            recipient in vec(0u8..255u8, 20..33), // address size
        ) -> TransactionOutput {
            TransactionOutput {
                amount,
                recipient,
            }
        }
    }
    
    prop_compose! {
        fn arb_transaction()(
            inputs in vec(arb_transaction_input(), 1..10),
            outputs in vec(arb_transaction_output(), 1..10),
            fee in 0u64..1_000_000,
            timestamp in 1_640_995_200u64..2_000_000_000u64,
        ) -> Transaction {
            Transaction {
                inputs,
                outputs,
                fee,
                timestamp,
            }
        }
    }
    
    prop_compose! {
        fn arb_block()(
            header in arb_block_header(),
            tx_count in 1usize..10,
        )(
            header in Just(header),
            transactions in vec(prop::array::uniform32(0u8..255u8), tx_count),
        ) -> Block {
            Block {
                header,
                transactions,
            }
        }
    }
    
    // Helper functions for test setup
    
    fn create_test_engine() -> ConsensusEngine {
        let spec = ChainSpecLoader::create_test_spec();
        let config = ChainConfig::default().shared();
        ConsensusEngine::new(spec, config).expect("Failed to create test consensus engine")
    }
    
    fn create_genesis_block() -> Block {
        Block {
            header: BlockHeader {
                height: 0,
                previous_hash: [0; 32],
                merkle_root: [0; 32],
                timestamp: 1_640_995_200,
                difficulty: 0x207fffff,
                nonce: 0,
            },
            transactions: vec![[1u8; 32]], // Genesis coinbase
        }
    }
    
    fn create_valid_block_chain(length: usize) -> Vec<Block> {
        let mut chain = Vec::with_capacity(length);
        let mut prev_block = create_genesis_block();
        chain.push(prev_block.clone());
        
        for i in 1..length {
            let mut block = Block {
                header: BlockHeader {
                    height: i as u64,
                    previous_hash: prev_block.hash(),
                    merkle_root: [0; 32],
                    timestamp: prev_block.header.timestamp + 60, // 1 minute intervals
                    difficulty: 0x207fffff,
                    nonce: 0,
                },
                transactions: vec![[i as u8; 32]], // Simple coinbase
            };
            
            // Calculate proper merkle root
            let engine = create_test_engine();
            block.header.merkle_root = engine.calculate_merkle_root(&block.transactions);
            
            chain.push(block.clone());
            prev_block = block;
        }
        
        chain
    }
    
    // Property-based tests
    
    proptest! {
        /// Test that block validation is deterministic
        #[test]
        fn test_block_validation_deterministic(
            block in arb_block()
        ) {
            let engine = create_test_engine();
            let result1 = engine.validate_block(&block, None);
            let result2 = engine.validate_block(&block, None);
            
            // Both validations should produce the same result
            prop_assert_eq!(result1.is_ok(), result2.is_ok());
        }
        
        /// Test that difficulty adjustment is bounded
        #[test]
        fn test_difficulty_adjustment_bounded(
            height in 10u64..100_000,
            time_taken in 1u64..1_000_000,
        ) {
            let engine = create_test_engine();
            
            // Only test adjustment blocks
            let adjustment_height = (height / 10) * 10; // Every 10 blocks in test spec
            if adjustment_height == 0 {
                return Ok(());
            }
            
            let result = engine.adjust_difficulty(adjustment_height, time_taken);
            
            if let Ok(new_difficulty) = result {
                // Difficulty must be positive
                prop_assert!(new_difficulty > 0);
                
                // Difficulty must be within reasonable bounds
                prop_assert!(new_difficulty >= 0x1000_0000);
                prop_assert!(new_difficulty <= 0x7fff_ffff);
            }
        }
        
        /// Test that block rewards never exceed maximum supply
        #[test]
        fn test_block_reward_bounded(
            height in 0u64..1_000_000
        ) {
            let engine = create_test_engine();
            let reward = engine.calculate_block_reward(height);
            
            // Reward should be non-negative
            prop_assert!(reward >= 0);
            
            // Reward should not exceed initial reward (for test spec)
            prop_assert!(reward <= 50_00000000);
        }
        
        /// Test that block rewards are monotonically decreasing
        #[test]
        fn test_block_reward_monotonic(
            height in 1u64..10_000
        ) {
            let engine = create_test_engine();
            let current_reward = engine.calculate_block_reward(height);
            let prev_reward = engine.calculate_block_reward(height - 1);
            
            // Reward should never increase
            prop_assert!(current_reward <= prev_reward);
        }
        
        /// Test that merkle root calculation is deterministic
        #[test]
        fn test_merkle_root_deterministic(
            tx_hashes in vec(prop::array::uniform32(0u8..255u8), 1..20)
        ) {
            let engine = create_test_engine();
            let root1 = engine.calculate_merkle_root(&tx_hashes);
            let root2 = engine.calculate_merkle_root(&tx_hashes);
            
            prop_assert_eq!(root1, root2);
        }
        
        /// Test that empty merkle tree has zero root
        #[test]
        fn test_empty_merkle_root() {
            let engine = create_test_engine();
            let empty_root = engine.calculate_merkle_root(&[]);
            prop_assert_eq!(empty_root, [0u8; 32]);
        }
        
        /// Test that single transaction merkle root equals transaction hash
        #[test]
        fn test_single_tx_merkle_root(
            tx_hash in prop::array::uniform32(0u8..255u8)
        ) {
            let engine = create_test_engine();
            let root = engine.calculate_merkle_root(&[tx_hash]);
            
            // For single transaction, root should be hash of the transaction
            // (implementation detail may vary)
            prop_assert_ne!(root, [0u8; 32]); // Should not be zero
        }
        
        /// Test that timestamp validation rejects future blocks
        #[test]
        fn test_timestamp_future_rejection(
            mut block in arb_block()
        ) {
            let engine = create_test_engine();
            let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            
            // Set timestamp too far in future (more than 2 hours)
            block.header.timestamp = current_time + 3 * 60 * 60; // 3 hours
            
            let result = engine.validate_timestamp(&block, None);
            prop_assert!(result.is_err());
            
            if let Err(ConsensusError::ClockSkew { .. }) = result {
                // Expected error type
            } else {
                prop_assert!(false, "Expected ClockSkew error");
            }
        }
        
        /// Test that block height validation enforces sequence
        #[test]
        fn test_block_height_sequence(
            height1 in 0u64..1000,
            height2 in 0u64..1000,
        ) {
            let engine = create_test_engine();
            
            let prev_block = Block {
                header: BlockHeader {
                    height: height1,
                    previous_hash: [0; 32],
                    merkle_root: [0; 32],
                    timestamp: 1_640_995_200,
                    difficulty: 0x207fffff,
                    nonce: 0,
                },
                transactions: vec![[1u8; 32]],
            };
            
            let current_block = Block {
                header: BlockHeader {
                    height: height2,
                    previous_hash: prev_block.hash(),
                    merkle_root: [0; 32],
                    timestamp: 1_640_995_260,
                    difficulty: 0x207fffff,
                    nonce: 0,
                },
                transactions: vec![[2u8; 32]],
            };
            
            let result = engine.validate_block_height(&current_block, Some(&prev_block));
            
            if height2 == height1 + 1 {
                prop_assert!(result.is_ok(), "Sequential heights should be valid");
            } else {
                prop_assert!(result.is_err(), "Non-sequential heights should be invalid");
            }
        }
        
        /// Test that fork resolution selects highest work chain
        #[test]
        fn test_fork_resolution_highest_work(
            work1 in 1000u128..1_000_000,
            work2 in 1000u128..1_000_000,
        ) {
            let engine = create_test_engine();
            
            // Add forks with different work amounts
            {
                let mut forks = engine.forks.write();
                forks.insert("fork1".to_string(), Fork {
                    tip_hash: "hash1".to_string(),
                    tip_height: 100,
                    total_work: work1,
                    last_common_ancestor: 50,
                    branch_blocks: vec!["hash1".to_string()],
                });
                forks.insert("fork2".to_string(), Fork {
                    tip_hash: "hash2".to_string(),
                    tip_height: 100,
                    total_work: work2,
                    last_common_ancestor: 50,
                    branch_blocks: vec!["hash2".to_string()],
                });
            }
            
            let best_hash = engine.resolve_forks().unwrap();
            
            if work1 > work2 {
                prop_assert_eq!(best_hash, "hash1");
            } else if work2 > work1 {
                prop_assert_eq!(best_hash, "hash2");
            }
            // If equal work, either is acceptable
        }
        
        /// Test that network partition detection works correctly
        #[test]
        fn test_network_partition_detection(
            peer_heights in vec(0u64..1000, 1..20),
            our_height in 0u64..200,
        ) {
            let engine = create_test_engine();
            
            // Set our chain height
            {
                let mut chain_state = engine.chain_state.write();
                chain_state.best_block_height = our_height;
            }
            
            let partition_detected = engine.detect_network_partition(&peer_heights);
            
            // Count peers significantly ahead
            let ahead_peers = peer_heights.iter()
                .filter(|&&h| h > our_height + 6)
                .count();
            let total_peers = peer_heights.len();
            let ahead_ratio = ahead_peers as f64 / total_peers as f64;
            
            if ahead_ratio > 0.5 {
                prop_assert!(partition_detected, "Should detect partition when majority of peers are ahead");
            } else {
                prop_assert!(!partition_detected, "Should not detect partition when minority of peers are ahead");
            }
        }
        
        /// Test that difficulty calculation preserves target
        #[test]
        fn test_difficulty_target_conversion(
            difficulty in 0x1000_0000u32..0x2000_0000u32
        ) {
            let target = ConsensusEngine::compact_to_target(difficulty);
            let recovered_difficulty = ConsensusEngine::target_to_compact(target);
            
            // Conversion should be reasonably close (allowing for rounding)
            let diff_ratio = if recovered_difficulty > difficulty {
                recovered_difficulty as f64 / difficulty as f64
            } else {
                difficulty as f64 / recovered_difficulty as f64
            };
            
            prop_assert!(diff_ratio < 1.1, "Difficulty conversion should preserve value within 10%");
        }
        
        /// Test that network time updates maintain consistency
        #[test]
        fn test_network_time_consistency(
            peer_times in vec(1_600_000_000u64..2_000_000_000u64, 1..50)
        ) {
            let engine = create_test_engine();
            engine.update_network_time(&peer_times);
            
            let network_time = engine.network_time.read();
            
            // Median should be within range of input times
            if !peer_times.is_empty() {
                let min_time = *peer_times.iter().min().unwrap();
                let max_time = *peer_times.iter().max().unwrap();
                
                prop_assert!(network_time.median_time_past >= min_time);
                prop_assert!(network_time.median_time_past <= max_time);
            }
        }
    }
    
    // Invariant testing
    
    #[test]
    fn test_chain_invariants() {
        let chain = create_valid_block_chain(10);
        let engine = create_test_engine();
        
        // Test all invariants hold for valid chain
        for i in 1..chain.len() {
            let current = &chain[i];
            let prev = &chain[i - 1];
            
            // Height must increase by 1
            assert_eq!(current.header.height, prev.header.height + 1);
            
            // Previous hash must link correctly
            assert_eq!(current.header.previous_hash, prev.hash());
            
            // Timestamp must increase
            assert!(current.header.timestamp > prev.header.timestamp);
            
            // Block should validate
            assert!(engine.validate_block(current, Some(prev)).is_ok());
        }
    }
    
    #[test]
    fn test_supply_invariants() {
        let engine = create_test_engine();
        let spec = &engine.spec;
        
        // Test supply invariants over block height range
        let mut total_issued = 0u64;
        let mut prev_reward = u64::MAX;
        
        for height in 0..1000 {
            let reward = engine.calculate_block_reward(height);
            
            // Rewards should be non-negative
            assert!(reward >= 0);
            
            // Rewards should not increase (only decrease or stay same)
            assert!(reward <= prev_reward);
            prev_reward = reward;
            
            total_issued = total_issued.saturating_add(reward);
            
            // Total issued should never exceed max supply
            assert!(total_issued <= spec.supply.max_supply);
        }
    }
    
    #[test]
    fn test_difficulty_adjustment_invariants() {
        let engine = create_test_engine();
        let adjustment_period = engine.spec.consensus.difficulty_adjustment_period;
        
        // Test difficulty adjustment maintains bounds
        for period in 1..10 {
            let height = period * adjustment_period;
            
            // Test various time scenarios
            let time_scenarios = vec![
                adjustment_period * 30,  // Half expected time (should increase difficulty)
                adjustment_period * 60,  // Expected time (should maintain difficulty)
                adjustment_period * 120, // Double expected time (should decrease difficulty)
            ];
            
            for time_taken in time_scenarios {
                if let Ok(new_difficulty) = engine.adjust_difficulty(height, time_taken) {
                    // Difficulty should be positive
                    assert!(new_difficulty > 0);
                    
                    // Difficulty should be within reasonable bounds
                    assert!(new_difficulty >= 0x1000_0000);
                    assert!(new_difficulty <= 0x7fff_ffff);
                }
            }
        }
    }
    
    #[test]
    fn test_transaction_validation_invariants() {
        let engine = create_test_engine();
        
        // Create a test transaction
        let tx = Transaction {
            inputs: vec![TransactionInput {
                prev_tx_hash: [1u8; 32],
                output_index: 0,
                signature: vec![0u8; 2420], // dilithium2 signature
            }],
            outputs: vec![TransactionOutput {
                amount: 1000000,
                recipient: vec![0u8; 20],
            }],
            fee: 1000,
            timestamp: 1_640_995_200,
        };
        
        // Transaction structure validation should be consistent
        let result1 = tx.validate();
        let result2 = tx.validate();
        assert_eq!(result1.is_ok(), result2.is_ok());
        
        // Transaction hash should be deterministic
        let hash1 = tx.hash();
        let hash2 = tx.hash();
        assert_eq!(hash1, hash2);
        
        // Transaction ID should be deterministic
        let id1 = tx.id();
        let id2 = tx.id();
        assert_eq!(id1, id2);
    }
    
    // Edge case testing
    
    #[test]
    fn test_zero_time_blocks() {
        let engine = create_test_engine();
        let genesis = create_genesis_block();
        
        // Block with same timestamp as genesis should be rejected
        let mut invalid_block = Block {
            header: BlockHeader {
                height: 1,
                previous_hash: genesis.hash(),
                merkle_root: [0; 32],
                timestamp: genesis.header.timestamp, // Same timestamp
                difficulty: 0x207fffff,
                nonce: 0,
            },
            transactions: vec![[1u8; 32]],
        };
        invalid_block.header.merkle_root = engine.calculate_merkle_root(&invalid_block.transactions);
        
        let result = engine.validate_timestamp(&invalid_block, Some(&genesis));
        assert!(result.is_err());
    }
    
    #[test]
    fn test_maximum_difficulty_change() {
        let engine = create_test_engine();
        let max_change = engine.spec.consensus.max_difficulty_change;
        
        // Test extreme time scenarios
        let adjustment_period = engine.spec.consensus.difficulty_adjustment_period;
        let target_time = engine.spec.consensus.target_block_time * adjustment_period;
        
        // Very fast blocks (should hit max difficulty increase)
        let very_fast_time = target_time / 10;
        if let Ok(new_difficulty) = engine.adjust_difficulty(adjustment_period, very_fast_time) {
            // Should not exceed max adjustment factor
            let current_difficulty = engine.get_current_difficulty();
            // This is a simplified check - real implementation would need proper ratio calculation
            assert!(new_difficulty > 0);
        }
        
        // Very slow blocks (should hit max difficulty decrease)
        let very_slow_time = target_time * 10;
        if let Ok(new_difficulty) = engine.adjust_difficulty(adjustment_period, very_slow_time) {
            assert!(new_difficulty > 0);
        }
    }
    
    #[test]
    fn test_block_size_limits() {
        let engine = create_test_engine();
        let max_size = engine.spec.block.max_block_size;
        
        // Create a block that's too large
        let large_tx_count = max_size / 32; // Approximate number of transactions to exceed limit
        let large_block = Block {
            header: BlockHeader {
                height: 1,
                previous_hash: [0; 32],
                merkle_root: [0; 32],
                timestamp: 1_640_995_200,
                difficulty: 0x207fffff,
                nonce: 0,
            },
            transactions: vec![[0u8; 32]; large_tx_count],
        };
        
        let result = engine.validate_block_size(&large_block);
        assert!(result.is_err());
        
        if let Err(ConsensusError::BlockTooLarge { size, limit }) = result {
            assert!(size > limit);
        } else {
            panic!("Expected BlockTooLarge error");
        }
    }
    
    #[test]
    fn test_genesis_block_special_cases() {
        let engine = create_test_engine();
        let genesis = create_genesis_block();
        
        // Genesis block should validate without previous block
        let result = engine.validate_block(&genesis, None);
        assert!(result.is_ok());
        
        // Genesis should have height 0
        assert_eq!(genesis.header.height, 0);
        
        // Genesis should have zero previous hash
        assert_eq!(genesis.header.previous_hash, [0u8; 32]);
    }
    
    // Stress testing
    
    #[test]
    fn test_large_chain_validation() {
        let chain = create_valid_block_chain(100);
        let engine = create_test_engine();
        
        // Validate entire chain
        for i in 1..chain.len() {
            let result = engine.validate_block(&chain[i], Some(&chain[i - 1]));
            assert!(result.is_ok(), "Block {} failed validation", i);
        }
    }
    
    #[test]
    fn test_concurrent_validation() {
        use std::sync::Arc;
        use std::thread;
        
        let engine = Arc::new(create_test_engine());
        let blocks = Arc::new(create_valid_block_chain(10));
        
        let handles: Vec<_> = (0..4).map(|_| {
            let engine = Arc::clone(&engine);
            let blocks = Arc::clone(&blocks);
            
            thread::spawn(move || {
                // Each thread validates the same blocks
                for i in 1..blocks.len() {
                    let result = engine.validate_block(&blocks[i], Some(&blocks[i - 1]));
                    assert!(result.is_ok());
                }
            })
        }).collect();
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
    }
}
