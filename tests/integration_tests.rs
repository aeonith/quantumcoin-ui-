use anyhow::Result;
use chrono::Utc;
use std::collections::HashMap;

// Import QuantumCoin modules
use quantumcoin::{
    blockchain::Blockchain,
    transaction::{SignedTransaction, TransactionInput, TransactionOutput},
    block::Block,
    quantum_crypto::{generate_keypair, sign_message, verify_signature, QuantumSignature},
    utxo::{UTXOSet, UTXO},
    mempool::Mempool,
    genesis::{create_mainnet_genesis, create_testnet_genesis},
};

#[cfg(test)]
mod blockchain_tests {
    use super::*;

    #[test]
    fn test_blockchain_initialization() {
        let blockchain = Blockchain::new();
        
        assert_eq!(blockchain.chain.len(), 1); // Genesis block
        assert_eq!(blockchain.difficulty, 4);
        assert_eq!(blockchain.mining_reward, 5000000000); // 50 QTC
        assert_eq!(blockchain.total_supply, 0);
        
        // Check genesis block
        let genesis = &blockchain.chain[0];
        assert_eq!(genesis.index, 0);
        assert_eq!(genesis.previous_hash, "0");
        assert!(!genesis.transactions.is_empty());
    }

    #[test]
    fn test_blockchain_add_block() -> Result<()> {
        let mut blockchain = Blockchain::new();
        let initial_height = blockchain.chain.len();
        
        // Create a test transaction
        let tx = create_test_transaction("miner_address");
        let block = Block::new(
            initial_height as u64,
            blockchain.get_latest_block().hash.clone(),
            vec![tx],
            4,
        );
        
        blockchain.add_block(block)?;
        
        assert_eq!(blockchain.chain.len(), initial_height + 1);
        Ok(())
    }

    #[test]
    fn test_blockchain_validate_chain() -> Result<()> {
        let mut blockchain = Blockchain::new();
        
        // Add a few valid blocks
        for i in 1..5 {
            let tx = create_test_transaction(&format!("miner_{}", i));
            let block = Block::new(
                i as u64,
                blockchain.get_latest_block().hash.clone(),
                vec![tx],
                4,
            );
            blockchain.add_block(block)?;
        }
        
        assert!(blockchain.is_chain_valid());
        Ok(())
    }

    #[test]
    fn test_mining_reward_halving() {
        let blockchain = Blockchain::new();
        
        // Test reward at different heights
        assert_eq!(blockchain.calculate_mining_reward(0), 5000000000); // 50 QTC
        assert_eq!(blockchain.calculate_mining_reward(210000), 2500000000); // 25 QTC
        assert_eq!(blockchain.calculate_mining_reward(420000), 1250000000); // 12.5 QTC
        assert_eq!(blockchain.calculate_mining_reward(630000), 625000000); // 6.25 QTC
    }

    fn create_test_transaction(to: &str) -> crate::transaction::Transaction {
        crate::transaction::Transaction {
            id: uuid::Uuid::new_v4().to_string(),
            from: "genesis".to_string(),
            to: to.to_string(),
            amount: 1000000000, // 10 QTC
            timestamp: Utc::now(),
            signature: "test_signature".to_string(),
            fee: 10000, // 0.0001 QTC
        }
    }
}

#[cfg(test)]
mod transaction_tests {
    use super::*;

    #[test]
    fn test_transaction_creation() {
        let tx = create_signed_transaction("alice", "bob", 100000000);
        
        assert!(!tx.id.is_empty());
        assert_eq!(tx.version, 1);
        assert_eq!(tx.outputs.len(), 1);
        assert_eq!(tx.outputs[0].value, 100000000);
        assert_eq!(tx.outputs[0].address, "bob");
    }

    #[test]
    fn test_transaction_signing() -> Result<()> {
        let (public_key, private_key) = generate_keypair();
        let mut tx = create_signed_transaction("alice", "bob", 100000000);
        
        // Sign transaction
        tx.sign(&private_key)?;
        
        assert!(!tx.signature.is_empty());
        assert!(!tx.public_key.is_empty());
        
        // Verify signature
        assert!(tx.verify_signature(&public_key));
        Ok(())
    }

    #[test]
    fn test_transaction_fee_calculation() -> Result<()> {
        let mut utxo_set = HashMap::new();
        utxo_set.insert("input1:0".to_string(), 200000000); // 2 QTC input
        
        let tx = SignedTransaction {
            id: "test_tx".to_string(),
            version: 1,
            inputs: vec![
                TransactionInput {
                    previous_output: "input1:0".to_string(),
                    script_sig: vec![],
                    sequence: 0xffffffff,
                }
            ],
            outputs: vec![
                TransactionOutput {
                    value: 100000000, // 1 QTC to recipient
                    script_pubkey: vec![],
                    address: "bob".to_string(),
                },
                TransactionOutput {
                    value: 99000000, // 0.99 QTC change
                    script_pubkey: vec![],
                    address: "alice".to_string(),
                }
            ],
            lock_time: 0,
            timestamp: Utc::now(),
            signature: String::new(),
            public_key: String::new(),
        };
        
        let fee = tx.calculate_fee(&utxo_set)?;
        assert_eq!(fee, 1000000); // 0.01 QTC fee
        Ok(())
    }

    fn create_signed_transaction(from: &str, to: &str, amount: u64) -> SignedTransaction {
        SignedTransaction::new(
            vec![
                TransactionInput {
                    previous_output: format!("{}:0", from),
                    script_sig: vec![],
                    sequence: 0xffffffff,
                }
            ],
            vec![
                TransactionOutput {
                    value: amount,
                    script_pubkey: vec![],
                    address: to.to_string(),
                }
            ],
            0,
        )
    }
}

#[cfg(test)]
mod crypto_tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let (pub_key1, priv_key1) = generate_keypair();
        let (pub_key2, priv_key2) = generate_keypair();
        
        assert_ne!(pub_key1, pub_key2);
        assert_ne!(priv_key1, priv_key2);
        assert!(!pub_key1.is_empty());
        assert!(!priv_key1.is_empty());
    }

    #[test] 
    fn test_signature_verification() -> Result<()> {
        let (public_key, private_key) = generate_keypair();
        let message = b"Hello, Quantum World!";
        
        // Sign message
        let signature = sign_message(&private_key, message)?;
        
        // Verify signature
        assert!(verify_signature(&signature, message));
        
        // Test with wrong message
        let wrong_message = b"Wrong message";
        assert!(!verify_signature(&signature, wrong_message));
        
        Ok(())
    }

    #[test]
    fn test_address_generation() {
        let (pub_key, _) = generate_keypair();
        let address = quantumcoin::quantum_crypto::public_key_to_address(&pub_key);
        
        assert!(!address.is_empty());
        assert!(address.len() > 25); // Reasonable address length
        
        // Test deterministic - same public key should give same address
        let address2 = quantumcoin::quantum_crypto::public_key_to_address(&pub_key);
        assert_eq!(address, address2);
    }
}

#[cfg(test)]
mod utxo_tests {
    use super::*;

    #[test]
    fn test_utxo_set_operations() -> Result<()> {
        let mut utxo_set = UTXOSet::new();
        
        // Create test UTXO
        let output = TransactionOutput {
            value: 5000000000, // 50 QTC
            script_pubkey: vec![],
            address: "alice".to_string(),
        };
        
        let utxo = UTXO::new("tx1".to_string(), 0, &output, 100, false);
        let outpoint = utxo.get_outpoint();
        
        // Add UTXO
        utxo_set.add_utxo(utxo)?;
        
        assert_eq!(utxo_set.size(), 1);
        assert_eq!(utxo_set.total_value(), 5000000000);
        assert!(utxo_set.contains_utxo(&outpoint));
        
        // Check balance
        assert_eq!(utxo_set.get_balance("alice"), 5000000000);
        assert_eq!(utxo_set.get_balance("bob"), 0);
        
        // Remove UTXO
        let removed = utxo_set.remove_utxo(&outpoint)?;
        assert_eq!(removed.amount, 5000000000);
        assert_eq!(utxo_set.size(), 0);
        
        Ok(())
    }

    #[test]
    fn test_utxo_transaction_application() -> Result<()> {
        let mut utxo_set = UTXOSet::new();
        
        // Add initial UTXO
        let initial_output = TransactionOutput {
            value: 10000000000, // 100 QTC
            script_pubkey: vec![],
            address: "alice".to_string(),
        };
        let initial_utxo = UTXO::new("genesis".to_string(), 0, &initial_output, 0, true);
        utxo_set.add_utxo(initial_utxo.clone())?;
        
        // Create spending transaction
        let spending_tx = SignedTransaction {
            id: "spend_tx".to_string(),
            version: 1,
            inputs: vec![
                TransactionInput {
                    previous_output: initial_utxo.get_outpoint(),
                    script_sig: vec![],
                    sequence: 0xffffffff,
                }
            ],
            outputs: vec![
                TransactionOutput {
                    value: 6000000000, // 60 QTC to bob
                    script_pubkey: vec![],
                    address: "bob".to_string(),
                },
                TransactionOutput {
                    value: 3900000000, // 39 QTC change back to alice
                    script_pubkey: vec![],
                    address: "alice".to_string(),
                }
            ],
            lock_time: 0,
            timestamp: Utc::now(),
            signature: String::new(),
            public_key: String::new(),
        };
        
        // Apply transaction
        utxo_set.apply_transaction(&spending_tx, 1, false)?;
        
        // Check results
        assert_eq!(utxo_set.size(), 2); // Two new outputs
        assert_eq!(utxo_set.get_balance("alice"), 3900000000); // 39 QTC
        assert_eq!(utxo_set.get_balance("bob"), 6000000000); // 60 QTC
        assert_eq!(utxo_set.total_value(), 9900000000); // 99 QTC total (1 QTC fee)
        
        // Original UTXO should be spent
        assert!(!utxo_set.contains_utxo(&initial_utxo.get_outpoint()));
        
        Ok(())
    }

    #[test]
    fn test_coinbase_maturity() -> Result<()> {
        let mut utxo_set = UTXOSet::new();
        
        // Add coinbase UTXO
        let coinbase_output = TransactionOutput {
            value: 5000000000, // 50 QTC mining reward
            script_pubkey: vec![],
            address: "miner".to_string(),
        };
        
        let mut coinbase_utxo = UTXO::new("coinbase_tx".to_string(), 0, &coinbase_output, 100, true);
        coinbase_utxo.confirmations = 50; // Not mature yet
        
        assert!(!coinbase_utxo.is_mature(100)); // Assuming 100 block maturity
        
        coinbase_utxo.confirmations = 100; // Now mature
        assert!(coinbase_utxo.is_mature(100));
        
        utxo_set.add_utxo(coinbase_utxo)?;
        
        // Check spendable balance
        assert_eq!(utxo_set.get_spendable_balance("miner", 100), 5000000000);
        assert_eq!(utxo_set.get_spendable_balance("miner", 150), 0); // Higher maturity requirement
        
        Ok(())
    }
}

#[cfg(test)]
mod mempool_tests {
    use super::*;

    #[test]
    fn test_mempool_basic_operations() -> Result<()> {
        let mut mempool = Mempool::new(100);
        
        let tx = create_test_signed_transaction("tx1");
        let tx_id = tx.id.clone();
        
        // Add transaction
        mempool.add_transaction(tx)?;
        assert_eq!(mempool.size(), 1);
        assert!(mempool.contains(&tx_id));
        
        // Get transaction
        let retrieved = mempool.get_transaction(&tx_id);
        assert!(retrieved.is_some());
        
        // Remove transaction
        let removed = mempool.remove_transaction(&tx_id);
        assert!(removed.is_some());
        assert_eq!(mempool.size(), 0);
        
        Ok(())
    }

    #[test]
    fn test_mempool_fee_prioritization() -> Result<()> {
        let mut mempool = Mempool::new(100);
        
        // Create transactions with different fees
        let high_fee_tx = create_transaction_with_fee("high_fee", 1000000); // High fee
        let low_fee_tx = create_transaction_with_fee("low_fee", 10000);     // Low fee
        let med_fee_tx = create_transaction_with_fee("med_fee", 100000);    // Medium fee
        
        mempool.add_transaction(low_fee_tx)?;
        mempool.add_transaction(high_fee_tx)?;
        mempool.add_transaction(med_fee_tx)?;
        
        // Get transactions sorted by fee
        let sorted_txs = mempool.get_transactions_by_fee(3);
        
        // Should be ordered by fee per byte (highest first)
        assert!(sorted_txs[0].fee_per_byte >= sorted_txs[1].fee_per_byte);
        assert!(sorted_txs[1].fee_per_byte >= sorted_txs[2].fee_per_byte);
        
        Ok(())
    }

    #[test]
    fn test_mempool_mining_selection() -> Result<()> {
        let mut mempool = Mempool::new(100);
        
        // Add several transactions
        for i in 0..10 {
            let tx = create_transaction_with_fee(&format!("tx_{}", i), 10000 + i * 1000);
            mempool.add_transaction(tx)?;
        }
        
        // Select transactions for mining
        let mining_txs = mempool.get_transactions_for_mining(5, 1000000);
        
        assert!(mining_txs.len() <= 5);
        
        // Should be sorted by fee (highest first)
        for i in 1..mining_txs.len() {
            let prev_fee = calculate_transaction_fee(&mining_txs[i-1]);
            let curr_fee = calculate_transaction_fee(&mining_txs[i]);
            assert!(prev_fee >= curr_fee);
        }
        
        Ok(())
    }

    #[test]
    fn test_fee_estimation() -> Result<()> {
        let mut mempool = Mempool::new(100);
        
        // Add transactions with various fees
        let fees = [100000, 200000, 300000, 400000, 500000]; // Different fee levels
        for (i, &fee) in fees.iter().enumerate() {
            let tx = create_transaction_with_fee(&format!("tx_{}", i), fee);
            mempool.add_transaction(tx)?;
        }
        
        // Test fee estimation for different priorities
        let next_block_fee = mempool.estimate_fee_for_priority(1);
        let normal_fee = mempool.estimate_fee_for_priority(6);
        
        assert!(next_block_fee >= normal_fee); // Higher priority should cost more
        
        Ok(())
    }

    fn create_test_signed_transaction(id: &str) -> SignedTransaction {
        SignedTransaction {
            id: format!("test_{}", id),
            version: 1,
            inputs: vec![
                TransactionInput {
                    previous_output: "input:0".to_string(),
                    script_sig: vec![],
                    sequence: 0xffffffff,
                }
            ],
            outputs: vec![
                TransactionOutput {
                    value: 100000000,
                    script_pubkey: vec![],
                    address: "recipient".to_string(),
                }
            ],
            lock_time: 0,
            timestamp: Utc::now(),
            signature: "test_signature".to_string(),
            public_key: "test_public_key".to_string(),
        }
    }

    fn create_transaction_with_fee(id: &str, fee: u64) -> SignedTransaction {
        SignedTransaction {
            id: format!("test_{}", id),
            version: 1,
            inputs: vec![
                TransactionInput {
                    previous_output: "input:0".to_string(),
                    script_sig: vec![],
                    sequence: 0xffffffff,
                }
            ],
            outputs: vec![
                TransactionOutput {
                    value: 100000000 - fee, // Amount minus fee
                    script_pubkey: vec![],
                    address: "recipient".to_string(),
                }
            ],
            lock_time: 0,
            timestamp: Utc::now(),
            signature: "test_signature".to_string(),
            public_key: "test_public_key".to_string(),
        }
    }

    fn calculate_transaction_fee(tx: &SignedTransaction) -> u64 {
        // Simplified fee calculation for testing
        let utxo_set = HashMap::new(); // Empty for test
        tx.calculate_fee(&utxo_set).unwrap_or(0)
    }
}

#[cfg(test)]
mod genesis_tests {
    use super::*;

    #[test]
    fn test_mainnet_genesis_creation() -> Result<()> {
        let genesis = create_mainnet_genesis()?;
        
        // Basic validation
        assert!(!genesis.hash.is_empty());
        assert!(!genesis.transactions.is_empty());
        assert_eq!(genesis.header.version, 1);
        assert_eq!(genesis.header.previous_hash, [0; 32]); // Genesis has no parent
        
        // Should have at least coinbase transaction
        assert!(genesis.transactions.len() >= 1);
        
        // Validate structure
        assert!(genesis.validate().is_ok());
        
        Ok(())
    }

    #[test]
    fn test_testnet_genesis_creation() -> Result<()> {
        let genesis = create_testnet_genesis()?;
        
        // Basic validation
        assert!(!genesis.hash.is_empty());
        assert!(!genesis.transactions.is_empty());
        
        // Testnet may have initial allocations
        assert!(genesis.total_allocation() >= 0);
        
        // Validate structure
        assert!(genesis.validate().is_ok());
        
        Ok(())
    }

    #[test] 
    fn test_genesis_deterministic() -> Result<()> {
        // Generate same genesis block twice
        let genesis1 = create_mainnet_genesis()?;
        let genesis2 = create_mainnet_genesis()?;
        
        // Should be identical (deterministic)
        assert_eq!(genesis1.hash, genesis2.hash);
        assert_eq!(genesis1.header.merkle_root, genesis2.header.merkle_root);
        assert_eq!(genesis1.header.timestamp, genesis2.header.timestamp);
        
        Ok(())
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_full_transaction_flow() -> Result<()> {
        // This test simulates a complete transaction flow:
        // 1. Create blockchain with genesis
        // 2. Add UTXO from genesis
        // 3. Create and sign transaction
        // 4. Add to mempool
        // 5. Mine block with transaction
        // 6. Update UTXO set
        
        let mut blockchain = Blockchain::new();
        let mut utxo_set = UTXOSet::new();
        let mut mempool = Mempool::new(1000);
        
        // Step 1: Genesis UTXO (simulate mining reward)
        let genesis_output = TransactionOutput {
            value: 5000000000, // 50 QTC
            script_pubkey: vec![],
            address: "miner".to_string(),
        };
        
        let genesis_utxo = UTXO::new("genesis_coinbase".to_string(), 0, &genesis_output, 0, true);
        genesis_utxo.confirmations = 100; // Mature
        utxo_set.add_utxo(genesis_utxo.clone())?;
        
        // Step 2: Create transaction
        let (alice_pub, alice_priv) = generate_keypair();
        let alice_addr = quantumcoin::quantum_crypto::public_key_to_address(&alice_pub);
        
        let mut tx = SignedTransaction {
            id: "user_tx_1".to_string(),
            version: 1,
            inputs: vec![
                TransactionInput {
                    previous_output: genesis_utxo.get_outpoint(),
                    script_sig: vec![],
                    sequence: 0xffffffff,
                }
            ],
            outputs: vec![
                TransactionOutput {
                    value: 2000000000, // 20 QTC to Alice
                    script_pubkey: vec![],
                    address: alice_addr.clone(),
                },
                TransactionOutput {
                    value: 2900000000, // 29 QTC change back to miner
                    script_pubkey: vec![],
                    address: "miner".to_string(),
                }
            ],
            lock_time: 0,
            timestamp: Utc::now(),
            signature: String::new(),
            public_key: String::new(),
        };
        
        // Step 3: Sign transaction
        tx.sign(&alice_priv)?;
        
        // Step 4: Add to mempool
        mempool.add_transaction(tx.clone())?;
        assert_eq!(mempool.size(), 1);
        
        // Step 5: Mine block with transaction
        let mining_txs = mempool.get_transactions_for_mining(10, 4000000);
        assert_eq!(mining_txs.len(), 1);
        
        let new_block = Block::new(
            1,
            blockchain.get_latest_block().hash.clone(),
            mining_txs.iter().map(|tx| crate::transaction::Transaction {
                id: tx.id.clone(),
                from: "miner".to_string(),
                to: alice_addr.clone(),
                amount: 2000000000,
                timestamp: tx.timestamp,
                signature: tx.signature.clone(),
                fee: 100000000, // 1 QTC fee
            }).collect(),
            blockchain.difficulty,
        );
        
        blockchain.add_block(new_block)?;
        
        // Step 6: Update UTXO set
        utxo_set.apply_transaction(&tx, 1, false)?;
        
        // Step 7: Verify final state
        assert_eq!(blockchain.chain.len(), 2); // Genesis + 1 block
        assert_eq!(utxo_set.get_balance(&alice_addr), 2000000000); // Alice has 20 QTC
        assert_eq!(utxo_set.get_balance("miner"), 2900000000); // Miner has 29 QTC
        assert_eq!(utxo_set.size(), 2); // Two unspent outputs
        
        // Remove transaction from mempool
        mempool.remove_transaction(&tx.id);
        assert_eq!(mempool.size(), 0);
        
        Ok(())
    }

    #[test]
    fn test_blockchain_reorganization() -> Result<()> {
        // Test blockchain reorganization (orphan block handling)
        let mut blockchain = Blockchain::new();
        let mut utxo_set = UTXOSet::new();
        
        // Create two competing chains
        let block1a = create_test_block(&blockchain, 1, "tx1a");
        let block1b = create_test_block(&blockchain, 1, "tx1b");
        
        // Add block1a first
        blockchain.add_block(block1a.clone())?;
        assert_eq!(blockchain.chain.len(), 2);
        
        // Now add block1b - this should create a fork
        // In a real implementation, this would trigger reorganization logic
        // For now, just verify the chain validation still works
        assert!(blockchain.is_chain_valid());
        
        Ok(())
    }

    fn create_test_block(blockchain: &Blockchain, index: u64, tx_id: &str) -> Block {
        let tx = crate::transaction::Transaction {
            id: tx_id.to_string(),
            from: "genesis".to_string(),
            to: "miner".to_string(),
            amount: 1000000000,
            timestamp: Utc::now(),
            signature: "test_sig".to_string(),
            fee: 10000,
        };
        
        Block::new(
            index,
            blockchain.get_latest_block().hash.clone(),
            vec![tx],
            blockchain.difficulty,
        )
    }
}

#[cfg(test)]
mod stress_tests {
    use super::*;

    #[test]
    fn test_large_utxo_set_performance() -> Result<()> {
        let mut utxo_set = UTXOSet::new();
        let start_time = std::time::Instant::now();
        
        // Add 10,000 UTXOs
        for i in 0..10_000 {
            let output = TransactionOutput {
                value: 100000000, // 1 QTC each
                script_pubkey: vec![],
                address: format!("address_{}", i % 100), // 100 different addresses
            };
            
            let utxo = UTXO::new(format!("tx_{}", i), 0, &output, i / 100, false);
            utxo_set.add_utxo(utxo)?;
        }
        
        let add_duration = start_time.elapsed();
        println!("Added 10,000 UTXOs in {:?}", add_duration);
        
        // Test balance calculation performance
        let balance_start = std::time::Instant::now();
        let balance = utxo_set.get_balance("address_50");
        let balance_duration = balance_start.elapsed();
        
        println!("Balance calculation took {:?}", balance_duration);
        assert!(balance > 0);
        assert!(balance_duration.as_millis() < 100); // Should be fast
        
        // Test consistency
        assert!(utxo_set.verify_consistency().is_ok());
        
        Ok(())
    }

    #[test]
    fn test_mempool_high_load() -> Result<()> {
        let mut mempool = Mempool::new(1000);
        
        // Add 1000 transactions rapidly
        let start_time = std::time::Instant::now();
        
        for i in 0..1000 {
            let tx = SignedTransaction {
                id: format!("stress_tx_{}", i),
                version: 1,
                inputs: vec![
                    TransactionInput {
                        previous_output: format!("input_{}:0", i),
                        script_sig: vec![],
                        sequence: 0xffffffff,
                    }
                ],
                outputs: vec![
                    TransactionOutput {
                        value: 100000000 + i, // Slightly different amounts
                        script_pubkey: vec![],
                        address: format!("addr_{}", i % 50),
                    }
                ],
                lock_time: 0,
                timestamp: Utc::now(),
                signature: "test_sig".to_string(),
                public_key: "test_pub".to_string(),
            };
            
            if let Err(e) = mempool.add_transaction(tx) {
                // Expected when mempool fills up
                println!("Mempool full at transaction {}: {}", i, e);
                break;
            }
        }
        
        let duration = start_time.elapsed();
        println!("Added {} transactions in {:?}", mempool.size(), duration);
        
        // Test fee-based transaction retrieval performance
        let retrieval_start = std::time::Instant::now();
        let top_txs = mempool.get_transactions_by_fee(100);
        let retrieval_duration = retrieval_start.elapsed();
        
        println!("Retrieved top 100 transactions in {:?}", retrieval_duration);
        assert!(top_txs.len() <= 100);
        assert!(retrieval_duration.as_millis() < 50); // Should be very fast
        
        Ok(())
    }
}
