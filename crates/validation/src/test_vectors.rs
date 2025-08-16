//! Test vectors for comprehensive transaction validation testing
//! 
//! Provides golden test cases including malicious/edge case transactions

use crate::{Transaction, TxInput, TxOutput, OutPoint};

/// Create a valid test transaction
pub fn create_valid_test_transaction() -> Transaction {
    Transaction {
        version: 1,
        inputs: vec![
            TxInput {
                previous_output: OutPoint {
                    txid: [1u8; 32],
                    vout: 0,
                },
                script_sig: create_valid_script_sig(),
                sequence: 0xfffffffe,
                witness: vec![],
            }
        ],
        outputs: vec![
            TxOutput {
                value: 5000000000, // 50 QTC
                script_pubkey: vec![0x76, 0xa9, 0x14], // P2PKH prefix
            }
        ],
        lock_time: 0,
        witness_flag: false,
    }
}

/// Create transaction with invalid signature
pub fn create_invalid_signature_transaction() -> Transaction {
    let mut tx = create_valid_test_transaction();
    tx.inputs[0].script_sig = vec![0x00; 10]; // Invalid script
    tx
}

/// Create double-spend transaction
pub fn create_double_spend_transaction() -> Transaction {
    Transaction {
        version: 1,
        inputs: vec![
            TxInput {
                previous_output: OutPoint {
                    txid: [2u8; 32], // Same UTXO as another tx
                    vout: 0,
                },
                script_sig: create_valid_script_sig(),
                sequence: 0xfffffffe,
                witness: vec![],
            }
        ],
        outputs: vec![
            TxOutput {
                value: 1000000000, // 10 QTC
                script_pubkey: vec![0x76, 0xa9, 0x14],
            }
        ],
        lock_time: 0,
        witness_flag: false,
    }
}

/// Create transaction with insufficient fees
pub fn create_low_fee_transaction() -> Transaction {
    Transaction {
        version: 1,
        inputs: vec![
            TxInput {
                previous_output: OutPoint {
                    txid: [3u8; 32],
                    vout: 0,
                },
                script_sig: create_valid_script_sig(),
                sequence: 0xfffffffe,
                witness: vec![],
            }
        ],
        outputs: vec![
            TxOutput {
                value: 4999999999, // Only 1 satoshi fee (too low)
                script_pubkey: vec![0x76, 0xa9, 0x14],
            }
        ],
        lock_time: 0,
        witness_flag: false,
    }
}

/// Create oversized transaction
pub fn create_oversized_transaction() -> Transaction {
    let large_script = vec![0u8; 200000]; // Very large script
    
    Transaction {
        version: 1,
        inputs: vec![
            TxInput {
                previous_output: OutPoint {
                    txid: [4u8; 32],
                    vout: 0,
                },
                script_sig: large_script.clone(),
                sequence: 0xfffffffe,
                witness: vec![],
            }
        ],
        outputs: vec![
            TxOutput {
                value: 1000000000,
                script_pubkey: large_script,
            }
        ],
        lock_time: 0,
        witness_flag: false,
    }
}

fn create_valid_script_sig() -> Vec<u8> {
    // Mock script sig with proper format but dummy data
    // In real implementation, this would be a valid Dilithium2 signature
    let mut script = Vec::new();
    
    // Signature length (2420 bytes for Dilithium2)
    script.extend_from_slice(&2420u16.to_le_bytes());
    // Mock signature data
    script.extend_from_slice(&[0x01u8; 2420]);
    
    // Public key length (1312 bytes for Dilithium2) 
    script.extend_from_slice(&1312u16.to_le_bytes());
    // Mock public key data
    script.extend_from_slice(&[0x02u8; 1312]);
    
    script
}
