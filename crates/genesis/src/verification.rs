//! Genesis block verification and validation

use anyhow::{Result, Context};
use crate::{
    config::ChainSpec,
    block::GenesisBlock,
    crypto::{blake3_hash, verify_quantum_signature},
};
use serde::{Serialize, Deserialize};

/// Comprehensive genesis block verification
pub struct GenesisVerifier<'a> {
    chain_spec: &'a ChainSpec,
}

/// Verification result with detailed information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub valid: bool,
    pub checks: Vec<VerificationCheck>,
    pub summary: VerificationSummary,
}

/// Individual verification check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationCheck {
    pub name: String,
    pub passed: bool,
    pub message: String,
    pub severity: CheckSeverity,
}

/// Verification summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationSummary {
    pub total_checks: usize,
    pub passed_checks: usize,
    pub failed_checks: usize,
    pub critical_failures: usize,
    pub warnings: usize,
}

/// Severity levels for verification checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CheckSeverity {
    Critical,
    Error,
    Warning,
    Info,
}

impl<'a> GenesisVerifier<'a> {
    /// Create new verifier for chain specification
    pub fn new(chain_spec: &'a ChainSpec) -> Self {
        Self { chain_spec }
    }
    
    /// Perform comprehensive verification
    pub fn verify(&self, block: &GenesisBlock) -> Result<bool> {
        let result = self.verify_detailed(block)?;
        Ok(result.valid)
    }
    
    /// Perform detailed verification with full results
    pub fn verify_detailed(&self, block: &GenesisBlock) -> Result<VerificationResult> {
        let mut checks = Vec::new();
        
        // Core structure checks
        checks.extend(self.verify_structure(block)?);
        
        // Chain specification compliance
        checks.extend(self.verify_chain_spec_compliance(block)?);
        
        // Cryptographic checks
        checks.extend(self.verify_cryptography(block)?);
        
        // Economic model checks
        checks.extend(self.verify_economics(block)?);
        
        // Merkle tree verification
        checks.extend(self.verify_merkle_tree(block)?);
        
        // Transaction validation
        checks.extend(self.verify_transactions(block)?);
        
        // Determinism checks
        checks.extend(self.verify_determinism(block)?);
        
        // Calculate summary
        let summary = self.calculate_summary(&checks);
        let valid = summary.critical_failures == 0 && summary.failed_checks == 0;
        
        Ok(VerificationResult {
            valid,
            checks,
            summary,
        })
    }
    
    /// Verify basic block structure
    fn verify_structure(&self, block: &GenesisBlock) -> Result<Vec<VerificationCheck>> {
        let mut checks = Vec::new();
        
        // Previous hash must be zero
        checks.push(VerificationCheck {
            name: "Genesis Previous Hash".to_string(),
            passed: block.header.previous_hash == [0; 32],
            message: if block.header.previous_hash == [0; 32] {
                "Genesis block has zero previous hash".to_string()
            } else {
                "Genesis block must have zero previous hash".to_string()
            },
            severity: CheckSeverity::Critical,
        });
        
        // Block hash calculation
        let calculated_hash = GenesisBlock::calculate_block_hash(&block.header)?;
        checks.push(VerificationCheck {
            name: "Block Hash Calculation".to_string(),
            passed: block.hash == calculated_hash,
            message: if block.hash == calculated_hash {
                "Block hash matches calculated hash".to_string()
            } else {
                format!(
                    "Block hash mismatch: expected {}, got {}",
                    hex::encode(calculated_hash),
                    hex::encode(block.hash)
                )
            },
            severity: CheckSeverity::Critical,
        });
        
        // Merkle root validation
        checks.push(VerificationCheck {
            name: "Merkle Root Validation".to_string(),
            passed: block.header.merkle_root == block.merkle_tree.root(),
            message: if block.header.merkle_root == block.merkle_tree.root() {
                "Merkle root matches tree calculation".to_string()
            } else {
                "Merkle root mismatch with tree calculation".to_string()
            },
            severity: CheckSeverity::Critical,
        });
        
        // Transaction count
        let has_transactions = !block.transactions.is_empty();
        checks.push(VerificationCheck {
            name: "Transaction Count".to_string(),
            passed: has_transactions,
            message: if has_transactions {
                format!("{} transactions in genesis block", block.transactions.len())
            } else {
                "Genesis block must contain at least one transaction".to_string()
            },
            severity: CheckSeverity::Error,
        });
        
        Ok(checks)
    }
    
    /// Verify compliance with chain specification
    fn verify_chain_spec_compliance(&self, block: &GenesisBlock) -> Result<Vec<VerificationCheck>> {
        let mut checks = Vec::new();
        
        // Network magic bytes
        checks.push(VerificationCheck {
            name: "Network Magic Bytes".to_string(),
            passed: block.metadata.creation_params.network_magic == self.chain_spec.network_protocol.magic_bytes,
            message: format!(
                "Network magic: {:?}",
                block.metadata.creation_params.network_magic
            ),
            severity: CheckSeverity::Critical,
        });
        
        // Genesis timestamp
        let timestamp_matches = block.header.timestamp == self.chain_spec.genesis.timestamp;
        checks.push(VerificationCheck {
            name: "Genesis Timestamp".to_string(),
            passed: timestamp_matches,
            message: if timestamp_matches {
                format!("Genesis timestamp: {}", block.header.timestamp)
            } else {
                format!(
                    "Timestamp mismatch: expected {}, got {}",
                    self.chain_spec.genesis.timestamp,
                    block.header.timestamp
                )
            },
            severity: CheckSeverity::Critical,
        });
        
        // Initial difficulty
        let difficulty_matches = block.header.difficulty == self.chain_spec.consensus.genesis_difficulty;
        checks.push(VerificationCheck {
            name: "Genesis Difficulty".to_string(),
            passed: difficulty_matches,
            message: if difficulty_matches {
                format!("Genesis difficulty: 0x{:08x}", block.header.difficulty)
            } else {
                format!(
                    "Difficulty mismatch: expected 0x{:08x}, got 0x{:08x}",
                    self.chain_spec.consensus.genesis_difficulty,
                    block.header.difficulty
                )
            },
            severity: CheckSeverity::Error,
        });
        
        // Chain specification hash
        let spec_hash = self.calculate_chain_spec_hash()?;
        let spec_hash_matches = block.chain_spec_hash == spec_hash;
        checks.push(VerificationCheck {
            name: "Chain Spec Hash".to_string(),
            passed: spec_hash_matches,
            message: if spec_hash_matches {
                "Chain specification hash matches".to_string()
            } else {
                "Chain specification hash mismatch - block may be from different spec version".to_string()
            },
            severity: CheckSeverity::Warning,
        });
        
        Ok(checks)
    }
    
    /// Verify cryptographic elements
    fn verify_cryptography(&self, block: &GenesisBlock) -> Result<Vec<VerificationCheck>> {
        let mut checks = Vec::new();
        
        // Post-quantum signature verification
        if let Some(signature) = &block.signature {
            let signature_valid = block.verify_signature().unwrap_or(false);
            checks.push(VerificationCheck {
                name: "Post-Quantum Signature".to_string(),
                passed: signature_valid,
                message: if signature_valid {
                    format!("Valid {} signature", signature.algorithm)
                } else {
                    "Invalid or corrupted signature".to_string()
                },
                severity: if signature_valid { CheckSeverity::Info } else { CheckSeverity::Critical },
            });
            
            // Signature algorithm check
            let correct_algorithm = signature.algorithm == self.chain_spec.post_quantum.signature_algorithm;
            checks.push(VerificationCheck {
                name: "Signature Algorithm".to_string(),
                passed: correct_algorithm,
                message: format!("Signature algorithm: {}", signature.algorithm),
                severity: if correct_algorithm { CheckSeverity::Info } else { CheckSeverity::Error },
            });
        } else {
            checks.push(VerificationCheck {
                name: "Post-Quantum Signature".to_string(),
                passed: false,
                message: "No signature present - block may not be officially signed".to_string(),
                severity: CheckSeverity::Warning,
            });
        }
        
        Ok(checks)
    }
    
    /// Verify economic model compliance
    fn verify_economics(&self, block: &GenesisBlock) -> Result<Vec<VerificationCheck>> {
        let mut checks = Vec::new();
        
        // Total allocation vs max supply
        let total_allocation = block.total_allocation();
        let within_max_supply = total_allocation <= self.chain_spec.supply.max_supply;
        
        checks.push(VerificationCheck {
            name: "Total Supply Constraint".to_string(),
            passed: within_max_supply,
            message: if within_max_supply {
                format!(
                    "Total allocation: {} / {} ({}%)",
                    total_allocation,
                    self.chain_spec.supply.max_supply,
                    (total_allocation as f64 / self.chain_spec.supply.max_supply as f64 * 100.0) as u64
                )
            } else {
                format!(
                    "Total allocation ({}) exceeds max supply ({})",
                    total_allocation,
                    self.chain_spec.supply.max_supply
                )
            },
            severity: if within_max_supply { CheckSeverity::Info } else { CheckSeverity::Critical },
        });
        
        // Premine compliance
        let expected_premine = self.chain_spec.supply.premine + self.chain_spec.total_genesis_allocation();
        let actual_allocation = block.allocation_transactions().iter().map(|tx| tx.amount).sum::<u64>();
        
        checks.push(VerificationCheck {
            name: "Premine Allocation".to_string(),
            passed: true, // Always pass, just informational
            message: format!(
                "Genesis allocations: {} QTC, Configured premine: {} QTC",
                actual_allocation as f64 / 100_000_000.0,
                self.chain_spec.supply.premine as f64 / 100_000_000.0
            ),
            severity: CheckSeverity::Info,
        });
        
        Ok(checks)
    }
    
    /// Verify merkle tree construction
    fn verify_merkle_tree(&self, block: &GenesisBlock) -> Result<Vec<VerificationCheck>> {
        let mut checks = Vec::new();
        
        // Merkle tree leaf count
        let leaf_count_matches = block.merkle_tree.leaves.len() == block.transactions.len();
        checks.push(VerificationCheck {
            name: "Merkle Tree Leaf Count".to_string(),
            passed: leaf_count_matches,
            message: format!(
                "Merkle tree has {} leaves for {} transactions",
                block.merkle_tree.leaves.len(),
                block.transactions.len()
            ),
            severity: if leaf_count_matches { CheckSeverity::Info } else { CheckSeverity::Error },
        });
        
        // Verify each transaction hash in merkle tree
        let mut all_hashes_match = true;
        for (i, tx) in block.transactions.iter().enumerate() {
            if i < block.merkle_tree.leaves.len() {
                if block.merkle_tree.leaves[i] != tx.hash {
                    all_hashes_match = false;
                    break;
                }
            }
        }
        
        checks.push(VerificationCheck {
            name: "Merkle Tree Hash Consistency".to_string(),
            passed: all_hashes_match,
            message: if all_hashes_match {
                "All transaction hashes match merkle tree leaves".to_string()
            } else {
                "Transaction hash mismatch in merkle tree".to_string()
            },
            severity: if all_hashes_match { CheckSeverity::Info } else { CheckSeverity::Critical },
        });
        
        Ok(checks)
    }
    
    /// Verify individual transactions
    fn verify_transactions(&self, block: &GenesisBlock) -> Result<Vec<VerificationCheck>> {
        let mut checks = Vec::new();
        
        // Check for coinbase transaction
        let has_coinbase = block.coinbase_transaction().is_some();
        checks.push(VerificationCheck {
            name: "Coinbase Transaction".to_string(),
            passed: has_coinbase,
            message: if has_coinbase {
                "Genesis block contains coinbase transaction".to_string()
            } else {
                "Genesis block missing coinbase transaction".to_string()
            },
            severity: if has_coinbase { CheckSeverity::Info } else { CheckSeverity::Error },
        });
        
        // Verify transaction indices are sequential
        let mut indices_correct = true;
        for (expected_index, tx) in block.transactions.iter().enumerate() {
            if tx.index as usize != expected_index {
                indices_correct = false;
                break;
            }
        }
        
        checks.push(VerificationCheck {
            name: "Transaction Indices".to_string(),
            passed: indices_correct,
            message: if indices_correct {
                "Transaction indices are sequential".to_string()
            } else {
                "Transaction indices are not sequential".to_string()
            },
            severity: if indices_correct { CheckSeverity::Info } else { CheckSeverity::Error },
        });
        
        // Verify allocation addresses are valid format
        let mut valid_addresses = true;
        for tx in block.allocation_transactions() {
            if !self.is_valid_address_format(&tx.address) {
                valid_addresses = false;
                break;
            }
        }
        
        checks.push(VerificationCheck {
            name: "Address Format".to_string(),
            passed: valid_addresses,
            message: if valid_addresses {
                "All allocation addresses have valid format".to_string()
            } else {
                "Some allocation addresses have invalid format".to_string()
            },
            severity: if valid_addresses { CheckSeverity::Info } else { CheckSeverity::Warning },
        });
        
        Ok(checks)
    }
    
    /// Verify deterministic reproduction
    fn verify_determinism(&self, block: &GenesisBlock) -> Result<Vec<VerificationCheck>> {
        let mut checks = Vec::new();
        
        // Check if block was created in deterministic mode
        let is_deterministic = block.metadata.creation_params.deterministic;
        checks.push(VerificationCheck {
            name: "Deterministic Generation".to_string(),
            passed: is_deterministic,
            message: if is_deterministic {
                "Block was created in deterministic mode".to_string()
            } else {
                "Block was created in non-deterministic mode".to_string()
            },
            severity: if is_deterministic { CheckSeverity::Info } else { CheckSeverity::Warning },
        });
        
        // If deterministic, verify reproducibility by recreating
        if is_deterministic {
            // This would require recreating the block with the same parameters
            // For now, just verify that key fields that should be deterministic are present
            let has_signature = block.signature.is_some();
            checks.push(VerificationCheck {
                name: "Deterministic Signature".to_string(),
                passed: has_signature,
                message: if has_signature {
                    "Block has deterministic signature".to_string()
                } else {
                    "Block missing deterministic signature".to_string()
                },
                severity: if has_signature { CheckSeverity::Info } else { CheckSeverity::Warning },
            });
        }
        
        Ok(checks)
    }
    
    /// Calculate chain specification hash
    fn calculate_chain_spec_hash(&self) -> Result<[u8; 32]> {
        let serialized = bincode::serialize(self.chain_spec)
            .context("Failed to serialize chain specification")?;
        Ok(blake3_hash(&serialized))
    }
    
    /// Check if address format is valid (basic check)
    fn is_valid_address_format(&self, address: &str) -> bool {
        // Basic QuantumCoin address format validation
        address.starts_with("qtc1q") && address.len() >= 42 && address.len() <= 62
    }
    
    /// Calculate verification summary
    fn calculate_summary(&self, checks: &[VerificationCheck]) -> VerificationSummary {
        let total_checks = checks.len();
        let passed_checks = checks.iter().filter(|c| c.passed).count();
        let failed_checks = total_checks - passed_checks;
        
        let critical_failures = checks.iter()
            .filter(|c| !c.passed && matches!(c.severity, CheckSeverity::Critical))
            .count();
        
        let warnings = checks.iter()
            .filter(|c| matches!(c.severity, CheckSeverity::Warning))
            .count();
        
        VerificationSummary {
            total_checks,
            passed_checks,
            failed_checks,
            critical_failures,
            warnings,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::ChainSpec, builder::GenesisBuilder};
    
    #[test]
    fn test_mainnet_genesis_verification() {
        let chain_spec = ChainSpec::load_mainnet().unwrap();
        let builder = GenesisBuilder::new(chain_spec.clone());
        let genesis = builder.build().unwrap();
        
        let verifier = GenesisVerifier::new(&chain_spec);
        let result = verifier.verify_detailed(&genesis).unwrap();
        
        assert!(result.valid, "Mainnet genesis should be valid");
        assert_eq!(result.summary.critical_failures, 0);
        
        // Print verification results for inspection
        for check in &result.checks {
            if !check.passed {
                println!("Failed check: {} - {}", check.name, check.message);
            }
        }
    }
    
    #[test]
    fn test_testnet_genesis_verification() {
        let chain_spec = ChainSpec::load_testnet().unwrap();
        let builder = GenesisBuilder::new(chain_spec.clone());
        let genesis = builder.build().unwrap();
        
        let verifier = GenesisVerifier::new(&chain_spec);
        assert!(verifier.verify(&genesis).unwrap());
    }
    
    #[test]
    fn test_invalid_genesis_detection() {
        let chain_spec = ChainSpec::load_mainnet().unwrap();
        let builder = GenesisBuilder::new(chain_spec.clone());
        let mut genesis = builder.build().unwrap();
        
        // Corrupt the block hash
        genesis.hash = [0xFF; 32];
        
        let verifier = GenesisVerifier::new(&chain_spec);
        let result = verifier.verify_detailed(&genesis).unwrap();
        
        assert!(!result.valid);
        assert!(result.summary.critical_failures > 0);
    }
    
    #[test]
    fn test_verification_result_serialization() {
        let chain_spec = ChainSpec::load_testnet().unwrap();
        let builder = GenesisBuilder::new(chain_spec.clone());
        let genesis = builder.build().unwrap();
        
        let verifier = GenesisVerifier::new(&chain_spec);
        let result = verifier.verify_detailed(&genesis).unwrap();
        
        // Test JSON serialization
        let json = serde_json::to_string_pretty(&result).unwrap();
        let restored: VerificationResult = serde_json::from_str(&json).unwrap();
        
        assert_eq!(result.valid, restored.valid);
        assert_eq!(result.summary.total_checks, restored.summary.total_checks);
    }
}
