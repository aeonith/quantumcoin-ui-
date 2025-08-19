#!/usr/bin/env cargo run --bin
//! Generate the official QuantumCoin genesis block

use anyhow::Result;
use hex;
use quantumcoin::genesis::{create_mainnet_genesis, create_testnet_genesis, GenesisVerifier};

fn main() -> Result<()> {
    println!("🪙 QuantumCoin Genesis Block Generator");
    println!("=====================================\n");

    // Generate mainnet genesis
    println!("Generating Mainnet Genesis Block...");
    match create_mainnet_genesis() {
        Ok(genesis) => {
            println!("✅ Mainnet genesis block created successfully!");
            println!("📊 Genesis Details:");
            println!("   Block Hash: {}", hex::encode(genesis.hash));
            println!("   Merkle Root: {}", hex::encode(genesis.header.merkle_root));
            println!("   Timestamp: {}", genesis.header.timestamp);
            println!("   Transactions: {}", genesis.transactions.len());
            println!("   Total Allocation: {} QTC", genesis.total_allocation() as f64 / 100_000_000.0);
            
            // Save to file
            let genesis_json = serde_json::to_string_pretty(&genesis)?;
            std::fs::write("mainnet_genesis.json", genesis_json)?;
            println!("   Saved to: mainnet_genesis.json");
            
            // Verify genesis block
            match genesis.validate() {
                Ok(_) => println!("✅ Genesis block validation passed!"),
                Err(e) => println!("❌ Genesis block validation failed: {}", e),
            }
        },
        Err(e) => {
            println!("❌ Failed to create mainnet genesis: {}", e);
            return Err(e);
        }
    }

    println!("\n" + "=".repeat(50).as_str());

    // Generate testnet genesis  
    println!("Generating Testnet Genesis Block...");
    match create_testnet_genesis() {
        Ok(genesis) => {
            println!("✅ Testnet genesis block created successfully!");
            println!("📊 Genesis Details:");
            println!("   Block Hash: {}", hex::encode(genesis.hash));
            println!("   Merkle Root: {}", hex::encode(genesis.header.merkle_root));
            println!("   Timestamp: {}", genesis.header.timestamp);
            println!("   Transactions: {}", genesis.transactions.len());
            println!("   Total Allocation: {} QTC", genesis.total_allocation() as f64 / 100_000_000.0);
            
            // Save to file
            let genesis_json = serde_json::to_string_pretty(&genesis)?;
            std::fs::write("testnet_genesis.json", genesis_json)?;
            println!("   Saved to: testnet_genesis.json");
            
            // Verify genesis block
            match genesis.validate() {
                Ok(_) => println!("✅ Genesis block validation passed!"),
                Err(e) => println!("❌ Genesis block validation failed: {}", e),
            }
        },
        Err(e) => {
            println!("❌ Failed to create testnet genesis: {}", e);
            return Err(e);
        }
    }

    println!("\n🎉 Genesis generation complete!");
    println!("📁 Files created:");
    println!("   - mainnet_genesis.json");
    println!("   - testnet_genesis.json");
    println!("\n🚀 QuantumCoin is ready for network deployment!");

    Ok(())
}
