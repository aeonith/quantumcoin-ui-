use crate::blockchain::{Blockchain, Transaction};
use pqcrypto_dilithium::dilithium3::{PublicKey, Signature, verify_detached};
use base64::decode;

pub fn validate_transaction(tx: &Transaction, blockchain: &Blockchain) -> bool {
    if tx.sender == "network" {
        return true;
    }

    let message = format!("{}:{}:{}", tx.sender, tx.recipient, tx.amount);
    let pk_bytes = match base64::decode(&tx.sender) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };

    let signature_bytes = match &tx.signature {
        Some(sig_b64) => match decode(sig_b64) {
            Ok(bytes) => bytes,
            Err(_) => return false,
        },
        None => return false,
    };

    let public_key = match PublicKey::from_bytes(&pk_bytes) {
        Ok(pk) => pk,
        Err(_) => return false,
    };

    let signature = match Signature::from_bytes(&signature_bytes) {
        Ok(sig) => sig,
        Err(_) => return false,
    };

    verify_detached(&signature, message.as_bytes(), &public_key).is_ok()
}

pub fn prevent_double_spend(tx: &Transaction, blockchain: &Blockchain) -> bool {
    let mut balance: f64 = 0.0;
    for block in &blockchain.blocks {
        for t in &block.transactions {
            if t.recipient == tx.sender {
                balance += t.amount;
            } else if t.sender == tx.sender {
                balance -= t.amount;
            }
        }
    }
    balance >= tx.amount
}