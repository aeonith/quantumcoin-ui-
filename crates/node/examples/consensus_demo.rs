//! QuantumCoin Consensus Engine Demo
//! 
//! This example demonstrates the production-grade consensus engine features:
//! - Chain specification loading
//! - Block validation with comprehensive checks
//! - Difficulty adjustment
//! - Fork resolution
//! - Network partition detection
//! - Property-based testing integration

use quantumcoin_node::{
    consensus::ConsensusSystem,
    consensus_engine::{ChainSpec, ConsensusError},
    chain_spec_loader::ChainSpecLoader,
    config::ChainConfig,
    block::{Block, BlockHeader},
    transaction::{Transaction, TransactionInput, TransactionOutput},
};
use anyhow::Result;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::init();
    
    println!("🚀 QuantumCoin Production Consensus Engine Demo");
    println!("================================================");
    
    // 1. Load chain specification
    println!("\n📋 Loading chain specification...");
    let spec = ChainSpecLoader::create_test_spec();
    println!("✅ Loaded {} v{}", spec.network.name, spec.network.version);
    println!("   Algorithm: {}", spec.consensus.algorithm);
    println!("   Target block time: {}s", spec.consensus.target_block_time);
    println!("   Max supply: {} coins", spec.supply.max_supply / 100_000_000);
    
    // 2. Initialize consensus system
    println!("\n🔧 Initializing consensus system...");
    let config = ChainConfig::default().shared();
    let consensus = ConsensusSystem::new(config, None)?;
    println!("✅ Consensus system initialized");
    
    // 3. Create and validate genesis block
    println!("\n🏗️  Creating genesis block...");
    let genesis = create_genesis_block();
    println!("   Genesis hash: {}", hex::encode(genesis.hash()));
    
    let validation_result = consensus.validate_block(&genesis, None);
    match validation_result {
        Ok(()) => println!("✅ Genesis block validated successfully"),
        Err(e) => println!("❌ Genesis validation failed: {}", e),
    }
    
    // 4. Create a valid block chain
    println!("\n⛓️  Building test blockchain...");
    let mut chain = vec![genesis.clone()];
    
    for i in 1..=5 {
        let prev_block = &chain[i - 1];
        let new_block = create_next_block(prev_block, i as u64)?;
        
        match consensus.validate_block(&new_block, Some(prev_block)) {
            Ok(()) => {
                println!("✅ Block {} validated", i);
                chain.push(new_block);
            }
            Err(e) => {
                println!("❌ Block {} validation failed: {}", i, e);
                break;
            }
        }
    }
    
    // 5. Test difficulty adjustment
    println!("\n⚖️  Testing difficulty adjustment...");
    let current_difficulty = consensus.get_current_difficulty();
    println!("   Current difficulty: 0x{:08x}", current_difficulty);
    
    // Simulate fast blocks (should increase difficulty)
    let fast_adjustment = consensus.adjust_difficulty(10, 300); // Half expected time
    match fast_adjustment {
        Ok(new_difficulty) => {
            println!("   Fast blocks difficulty: 0x{:08x}", new_difficulty);
        }
        Err(e) => println!("   Difficulty adjustment error: {}", e),
    }
    
    // Simulate slow blocks (should decrease difficulty)
    let slow_adjustment = consensus.adjust_difficulty(20, 1200); // Double expected time
    match slow_adjustment {
        Ok(new_difficulty) => {
            println!("   Slow blocks difficulty: 0x{:08x}", new_difficulty);
        }
        Err(e) => println!("   Difficulty adjustment error: {}", e),
    }
    
    // 6. Test fork resolution
    println!("\n🍴 Testing fork resolution...");
    demonstrate_fork_resolution(&consensus);
    
    // 7. Test network partition detection
    println!("\n🌐 Testing network partition detection...");
    demonstrate_network_partition(&consensus);
    
    // 8. Test block reward calculation
    println!("\n💰 Testing block reward schedule...");
    demonstrate_reward_schedule(&consensus);
    
    // 9. Health check
    println!("\n🏥 System health check...");
    let health = consensus.health_check()?;
    println!("   Network: {} v{}", health.network_name, health.version);
    println!("   Chain height: {}", health.chain_height);
    println!("   Best block: {}", health.best_block_hash);
    println!("   Total work: {}", health.total_work);
    println!("   Current difficulty: 0x{:08x}", health.current_difficulty);
    
    println!("\n🎉 Demo completed successfully!");
    Ok(())
}

fn create_genesis_block() -> Block {
    Block {
        header: BlockHeader {
            height: 0,
            previous_hash: [0; 32],
            merkle_root: [0; 32],
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            difficulty: 0x207fffff, // Test difficulty
            nonce: 0,
        },
        transactions: vec![[1u8; 32]], // Genesis coinbase transaction
    }
}

fn create_next_block(prev_block: &Block, height: u64) -> Result<Block> {
    let mut block = Block {
        header: BlockHeader {
            height,
            previous_hash: prev_block.hash(),
            merkle_root: [0; 32],
            timestamp: prev_block.header.timestamp + 60, // 1 minute later
            difficulty: prev_block.header.difficulty,
            nonce: 0,
        },
        transactions: vec![[height as u8; 32]], // Simple transaction
    };
    
    // Calculate proper merkle root (simplified)
    // In a real implementation, this would calculate from actual transactions
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    for tx_hash in &block.transactions {
        hasher.update(tx_hash);
    }
    block.header.merkle_root.copy_from_slice(&hasher.finalize()[..32]);
    
    Ok(block)
}

fn demonstrate_fork_resolution(consensus: &ConsensusSystem) {
    let fork_detected = consensus.resolve_forks();
    match fork_detected {
        Ok(best_hash) => {
            println!("   Best chain tip: {}", best_hash);
        }
        Err(e) => {
            println!("   No active forks: {}", e);
        }
    }
}

fn demonstrate_network_partition(consensus: &ConsensusSystem) {
    // Test normal network state
    let normal_peers = vec![100, 101, 99, 100, 102];
    let partition1 = consensus.detect_network_partition(&normal_peers);
    println!("   Normal network partition detected: {}", partition1);
    
    // Test partition state (peers much further ahead)
    let partition_peers = vec![200, 201, 199, 200, 202];
    let partition2 = consensus.detect_network_partition(&partition_peers);
    println!("   Partition network partition detected: {}", partition2);
}

fn demonstrate_reward_schedule(consensus: &ConsensusSystem) {
    println!("   Block reward schedule:");
    for height in [0, 100, 200, 300, 400, 500] {
        let reward = consensus.calculate_block_reward(height);
        let qtc_amount = reward as f64 / 100_000_000.0;
        println!("     Height {:3}: {:.8} QTC", height, qtc_amount);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    
    #[tokio::test]
    async fn test_consensus_demo_flow() {
        // Ensure the demo runs without errors
        let result = main().await;
        assert!(result.is_ok());
    }
    
    proptest! {
        #[test]
        fn test_block_creation_properties(
            height in 1u64..1000,
            time_offset in 60u64..3600,
        ) {
            let genesis = create_genesis_block();
            let result = create_next_block(&genesis, height);
            
            prop_assert!(result.is_ok());
            
            let block = result.unwrap();
            prop_assert_eq!(block.header.height, height);
            prop_assert_eq!(block.header.previous_hash, genesis.hash());
            prop_assert!(block.header.timestamp > genesis.header.timestamp);
        }
    }
}
