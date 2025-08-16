//! Integration tests for the production consensus engine
//! 
//! These tests validate that the entire consensus system works correctly
//! when integrated with the blockchain, including edge cases and stress testing.

use quantumcoin_node::{
    consensus::{ConsensusSystem, ConsensusError},
    consensus_engine::{ChainSpec, ConsensusEngine},
    chain_spec_loader::ChainSpecLoader,
    config::ChainConfig,
    block::{Block, BlockHeader},
    transaction::{Transaction, TransactionInput, TransactionOutput},
};
use anyhow::Result;
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::NamedTempFile;
use std::io::Write;

// Helper function to create a minimal valid chain spec
fn create_test_chain_spec_file() -> NamedTempFile {
    let mut temp_file = NamedTempFile::new().unwrap();
    let toml_content = r#"
[network]
name = "quantumcoin-integration-test"
symbol = "QTC-INT"
decimals = 8
version = "1.0.0-integration"

[consensus]
algorithm = "proof_of_work"
hash_function = "blake3"
target_block_time = 60
difficulty_adjustment_period = 10
max_difficulty_change = 4.0
genesis_difficulty = "0x207fffff"

[supply]
max_supply = 1000000000000000
initial_reward = 5000000000
halving_interval = 100
premine = 0

[transaction]
max_tx_size = 50000
min_tx_fee = 100
dust_threshold = 546
max_inputs_per_tx = 100
max_outputs_per_tx = 100
signature_hash_type = "dilithium2"

[block]
max_block_size = 1000000
max_block_weight = 1000000
coinbase_maturity = 10
max_reorg_depth = 3

[cryptography]
address_version = 111
private_key_version = 239
checksum_algorithm = "blake3"
signature_scheme = "dilithium2"
hash_algorithm = "blake3_256"

[network_protocol]
magic_bytes = [81, 84, 67, 73]
protocol_version = 70015
services = 1
default_port = 18333
max_connections = 125
connection_timeout = 5

[fees]
min_relay_fee = 100
increment_fee = 100
dust_relay_fee = 300
max_fee_rate = 1000000

[mempool]
max_size = 100000000
max_tx_count = 10000
expiry_time = 3600
replacement_enabled = true

[mining]
coinbase_flags = "QuantumCoin-Integration/1.0"
extra_nonce_placeholder = 8
witness_commitment_pos = 0

[governance]
bip9_activation_threshold = 8
bip9_min_activation_height = 0
lock_in_period = 10
timeout_period = 50

[checkpoints]
genesis = "0000000000000000000000000000000000000000000000000000000000000000"

[economic_model]
inflation_schedule = [
    { height = 0, reward = 5000000000 },
    { height = 100, reward = 2500000000 },
    { height = 200, reward = 1250000000 },
]

[post_quantum]
signature_algorithm = "dilithium2"
public_key_size = 1312
private_key_size = 2528
signature_size = 2420
security_level = 2

[ai_integration]
enabled = false
telemetry_endpoint = "https://test.example.com/telemetry"
anomaly_detection = false
fee_prediction = false
network_analysis = false

[metadata]
specification_version = "1.0.0"
created_at = "2025-01-15T00:00:00Z"
authors = ["integration-test"]
license = "MIT"
repository = "https://github.com/test/integration"
documentation = "https://docs.test.com"

[verification]
spec_hash = "blake3:integration-test"
genesis_hash = "blake3:integration-test"
checksum = "blake3:integration-test"
"#;
    
    temp_file.write_all(toml_content.as_bytes()).unwrap();
    temp_file
}

fn create_test_genesis_block() -> Block {
    Block {
        header: BlockHeader {
            height: 0,
            previous_hash: [0; 32],
            merkle_root: [1; 32], // Simple merkle root
            timestamp: 1_640_995_200, // Fixed timestamp for determinism
            difficulty: 0x207fffff, // Test difficulty
            nonce: 0,
        },
        transactions: vec![[1u8; 32]], // Genesis coinbase
    }
}

fn create_test_block(prev_block: &Block, height: u64) -> Block {
    let mut block = Block {
        header: BlockHeader {
            height,
            previous_hash: prev_block.hash(),
            merkle_root: [0; 32],
            timestamp: prev_block.header.timestamp + 60, // 1 minute intervals
            difficulty: prev_block.header.difficulty,
            nonce: 0,
        },
        transactions: vec![[height as u8; 32]], // Simple transaction
    };
    
    // Calculate proper merkle root (simplified for testing)
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    for tx_hash in &block.transactions {
        hasher.update(tx_hash);
    }
    block.header.merkle_root.copy_from_slice(&hasher.finalize()[..32]);
    
    block
}

#[tokio::test]
async fn test_consensus_system_initialization() {
    let config = ChainConfig::default().shared();
    let consensus = ConsensusSystem::new(config, None).unwrap();
    
    // Verify system initialized correctly
    let chain_state = consensus.get_chain_state();
    assert_eq!(chain_state.best_block_height, 0);
    assert!(consensus.get_current_difficulty() > 0);
    
    let health = consensus.health_check().unwrap();
    assert_eq!(health.network_name, "quantumcoin-test");
    assert!(health.max_supply > 0);
}

#[tokio::test]
async fn test_chain_spec_loading_and_validation() {
    let spec_file = create_test_chain_spec_file();
    let config = ChainConfig::default().shared();
    
    let consensus = ConsensusSystem::new(
        config,
        Some(spec_file.path().to_str().unwrap())
    ).unwrap();
    
    let spec = consensus.get_chain_spec();
    assert_eq!(spec.network.name, "quantumcoin-integration-test");
    assert_eq!(spec.consensus.target_block_time, 60);
    assert_eq!(spec.consensus.difficulty_adjustment_period, 10);
    assert_eq!(spec.supply.halving_interval, 100);
}

#[tokio::test]
async fn test_full_blockchain_validation() {
    let config = ChainConfig::default().shared();
    let consensus = ConsensusSystem::new(config, None).unwrap();
    
    // Create a chain of valid blocks
    let mut chain = Vec::new();
    let genesis = create_test_genesis_block();
    chain.push(genesis.clone());
    
    // Build a 10-block chain
    let mut prev_block = genesis;
    for i in 1..=10 {
        let block = create_test_block(&prev_block, i);
        
        // Each block should validate individually
        let result = consensus.validate_block(&block, Some(&prev_block));
        assert!(result.is_ok(), "Block {} failed validation: {:?}", i, result);
        
        chain.push(block.clone());
        prev_block = block;
    }
    
    // Validate entire chain at once
    let chain_result = consensus.validate_chain(&chain);
    assert!(chain_result.is_ok(), "Full chain validation failed: {:?}", chain_result);
}

#[tokio::test]
async fn test_difficulty_adjustment_integration() {
    let config = ChainConfig::default().shared();
    let consensus = ConsensusSystem::new(config, None).unwrap();
    
    let initial_difficulty = consensus.get_current_difficulty();
    
    // Test difficulty adjustment at interval boundary
    let adjustment_height = 10; // Every 10 blocks in test spec
    
    // Fast blocks should increase difficulty
    let fast_result = consensus.adjust_difficulty(adjustment_height, 300); // Half expected time
    assert!(fast_result.is_ok());
    
    // Slow blocks should decrease difficulty
    let slow_result = consensus.adjust_difficulty(adjustment_height * 2, 1200); // Double expected time
    assert!(slow_result.is_ok());
    
    // Non-adjustment heights should return current difficulty
    let non_adjustment_result = consensus.adjust_difficulty(5, 600);
    assert!(non_adjustment_result.is_ok());
    assert_eq!(non_adjustment_result.unwrap(), initial_difficulty);
}

#[tokio::test]
async fn test_block_reward_schedule_integration() {
    let config = ChainConfig::default().shared();
    let consensus = ConsensusSystem::new(config, None).unwrap();
    
    // Test reward schedule over several halvings
    let test_heights = [0, 50, 100, 150, 200, 250, 300];
    let mut prev_reward = u64::MAX;
    
    for &height in &test_heights {
        let reward = consensus.calculate_block_reward(height);
        
        // Rewards should be non-negative
        assert!(reward >= 0, "Reward at height {} is negative", height);
        
        // Rewards should decrease or stay same (halving schedule)
        assert!(reward <= prev_reward, 
                "Reward increased from {} to {} at height {}", 
                prev_reward, reward, height);
        
        prev_reward = reward;
    }
}

#[tokio::test]
async fn test_network_partition_detection_integration() {
    let config = ChainConfig::default().shared();
    let consensus = ConsensusSystem::new(config, None).unwrap();
    
    // Simulate various network scenarios
    let scenarios = vec![
        (vec![100, 101, 99, 100, 102], false, "Normal network"),
        (vec![200, 201, 199, 200, 202], true, "Majority partition"),
        (vec![50, 51, 49, 50, 52], false, "Behind but not partitioned"),
        (vec![500], true, "Single far-ahead peer"),
        (vec![], false, "No peers"),
    ];
    
    for (peer_heights, expected_partition, description) in scenarios {
        let detected = consensus.detect_network_partition(&peer_heights);
        assert_eq!(detected, expected_partition, 
                   "Partition detection failed for scenario: {}", description);
    }
}

#[tokio::test]
async fn test_fork_resolution_integration() {
    let config = ChainConfig::default().shared();
    let consensus = ConsensusSystem::new(config, None).unwrap();
    
    // Test fork resolution with no active forks
    let result = consensus.resolve_forks();
    assert!(result.is_ok(), "Fork resolution should work with no active forks");
}

#[tokio::test]
async fn test_invalid_block_rejection() {
    let config = ChainConfig::default().shared();
    let consensus = ConsensusSystem::new(config, None).unwrap();
    
    let genesis = create_test_genesis_block();
    let mut invalid_block = create_test_block(&genesis, 1);
    
    // Test invalid height
    invalid_block.header.height = 5; // Skip heights 2, 3, 4
    let result = consensus.validate_block(&invalid_block, Some(&genesis));
    assert!(result.is_err(), "Should reject block with invalid height");
    
    // Test invalid previous hash
    invalid_block.header.height = 1;
    invalid_block.header.previous_hash = [0xff; 32]; // Wrong previous hash
    let result = consensus.validate_block(&invalid_block, Some(&genesis));
    assert!(result.is_err(), "Should reject block with invalid previous hash");
    
    // Test timestamp too far in future
    invalid_block.header.previous_hash = genesis.hash();
    invalid_block.header.timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() + 3 * 60 * 60; // 3 hours in future
    let result = consensus.validate_block(&invalid_block, Some(&genesis));
    assert!(result.is_err(), "Should reject block with future timestamp");
}

#[tokio::test]
async fn test_transaction_validation_integration() {
    let config = ChainConfig::default().shared();
    let consensus = ConsensusSystem::new(config, None).unwrap();
    
    // Create a valid transaction structure
    let tx = Transaction {
        inputs: vec![TransactionInput {
            prev_tx_hash: [1u8; 32],
            output_index: 0,
            signature: vec![0u8; 100], // Simplified signature
        }],
        outputs: vec![TransactionOutput {
            amount: 1000000,
            recipient: vec![0u8; 20],
        }],
        fee: 1000,
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
    };
    
    // Basic transaction structure should validate
    let result = consensus.validate_transaction(&tx);
    assert!(result.is_ok(), "Valid transaction should pass basic validation");
    
    // Create invalid transaction (no inputs)
    let invalid_tx = Transaction {
        inputs: vec![], // No inputs
        outputs: vec![TransactionOutput {
            amount: 1000000,
            recipient: vec![0u8; 20],
        }],
        fee: 1000,
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
    };
    
    let result = consensus.validate_transaction(&invalid_tx);
    assert!(result.is_err(), "Transaction with no inputs should be invalid");
}

#[tokio::test]
async fn test_consensus_system_health_check() {
    let config = ChainConfig::default().shared();
    let consensus = ConsensusSystem::new(config, None).unwrap();
    
    let health = consensus.health_check().unwrap();
    
    // Verify all health check fields
    assert_eq!(health.chain_height, 0); // Initial height
    assert!(!health.best_block_hash.is_empty());
    assert_eq!(health.total_work, 0); // Initial work
    assert!(health.current_difficulty > 0);
    assert!(!health.network_name.is_empty());
    assert!(!health.version.is_empty());
    assert_eq!(health.consensus_algorithm, "proof_of_work");
    assert!(health.target_block_time > 0);
    assert!(health.max_supply > 0);
}

#[tokio::test]
async fn test_concurrent_validation() {
    use std::sync::Arc;
    use tokio::task;
    
    let config = ChainConfig::default().shared();
    let consensus = Arc::new(ConsensusSystem::new(config, None).unwrap());
    
    // Create test blocks
    let genesis = create_test_genesis_block();
    let block1 = create_test_block(&genesis, 1);
    let block2 = create_test_block(&block1, 2);
    
    // Run concurrent validations
    let consensus_clone1 = Arc::clone(&consensus);
    let genesis_clone1 = genesis.clone();
    let block1_clone1 = block1.clone();
    let handle1 = task::spawn(async move {
        consensus_clone1.validate_block(&block1_clone1, Some(&genesis_clone1))
    });
    
    let consensus_clone2 = Arc::clone(&consensus);
    let block1_clone2 = block1.clone();
    let block2_clone = block2.clone();
    let handle2 = task::spawn(async move {
        consensus_clone2.validate_block(&block2_clone, Some(&block1_clone2))
    });
    
    // Both validations should succeed
    let result1 = handle1.await.unwrap();
    let result2 = handle2.await.unwrap();
    
    assert!(result1.is_ok(), "Concurrent validation 1 failed");
    assert!(result2.is_ok(), "Concurrent validation 2 failed");
}

#[tokio::test]
async fn test_large_blockchain_validation() {
    let config = ChainConfig::default().shared();
    let consensus = ConsensusSystem::new(config, None).unwrap();
    
    // Create a longer chain for stress testing
    const CHAIN_LENGTH: usize = 50;
    let mut chain = Vec::with_capacity(CHAIN_LENGTH);
    
    let genesis = create_test_genesis_block();
    chain.push(genesis.clone());
    
    let mut prev_block = genesis;
    for i in 1..CHAIN_LENGTH {
        let block = create_test_block(&prev_block, i as u64);
        chain.push(block.clone());
        prev_block = block;
    }
    
    // Validate entire long chain
    let start_time = std::time::Instant::now();
    let result = consensus.validate_chain(&chain);
    let validation_time = start_time.elapsed();
    
    assert!(result.is_ok(), "Large chain validation failed: {:?}", result);
    println!("Validated {} blocks in {:?}", CHAIN_LENGTH, validation_time);
    
    // Performance check - should validate reasonably quickly
    assert!(validation_time.as_millis() < 1000, 
            "Chain validation took too long: {:?}", validation_time);
}

#[tokio::test]
async fn test_error_handling_and_recovery() {
    let config = ChainConfig::default().shared();
    let consensus = ConsensusSystem::new(config, None).unwrap();
    
    // Test that system continues working after errors
    let genesis = create_test_genesis_block();
    
    // Try to validate an invalid block
    let mut invalid_block = create_test_block(&genesis, 1);
    invalid_block.header.height = 999; // Wrong height
    
    let error_result = consensus.validate_block(&invalid_block, Some(&genesis));
    assert!(error_result.is_err());
    
    // System should still work after error
    let valid_block = create_test_block(&genesis, 1);
    let success_result = consensus.validate_block(&valid_block, Some(&genesis));
    assert!(success_result.is_ok(), "System should recover after error");
    
    // Health check should still work
    let health = consensus.health_check().unwrap();
    assert!(!health.network_name.is_empty());
}

#[tokio::test]
async fn test_edge_case_scenarios() {
    let config = ChainConfig::default().shared();
    let consensus = ConsensusSystem::new(config, None).unwrap();
    
    // Test empty blockchain validation
    let empty_chain: Vec<Block> = vec![];
    let result = consensus.validate_chain(&empty_chain);
    assert!(result.is_ok(), "Empty chain should validate successfully");
    
    // Test single block validation (genesis only)
    let genesis = create_test_genesis_block();
    let single_block_chain = vec![genesis.clone()];
    let result = consensus.validate_chain(&single_block_chain);
    assert!(result.is_ok(), "Single block chain should validate successfully");
    
    // Test network time update with empty peer times
    consensus.update_network_time(&[]);
    
    // Test network partition detection with empty peer list
    let partition = consensus.detect_network_partition(&[]);
    assert!(!partition, "Empty peer list should not indicate partition");
}
